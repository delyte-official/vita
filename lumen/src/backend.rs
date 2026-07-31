use std::path::Path;
use std::process::Command;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::passes::PassBuilderOptions;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::types::{IntType, StructType};
use inkwell::values::{FunctionValue, IntValue, StructValue};
use inkwell::{IntPredicate, OptimizationLevel};

use crate::middle_end::{IRProgram, Instruction, Value};

#[derive(Clone, Copy)]
enum RtValue<'ctx> {
    Int(IntValue<'ctx>),
    Struct(StructValue<'ctx>),
}

fn struct_type_for<'ctx>(context: &'ctx Context, name: &str, i32_type: IntType<'ctx>) -> Option<StructType<'ctx>> {
    match name {
        "Rectangle" => Some(context.struct_type(&[i32_type.into(), i32_type.into()], false)),
        _ => None,
    }
}

fn resolve<'ctx>(v: Value, temps: &[Option<RtValue<'ctx>>], locals: &[Option<RtValue<'ctx>>], i32_type: IntType<'ctx>) -> RtValue<'ctx> {
    match v {
        Value::Const(n) => RtValue::Int(i32_type.const_int(n as u64, true)),
        Value::Temp(i) => temps[i].expect("temp used before being computed"),
        Value::Var(slot) => locals[slot].expect("variable used before being declared"),
    }
}

fn resolve_int<'ctx>(v: Value, temps: &[Option<RtValue<'ctx>>], locals: &[Option<RtValue<'ctx>>], i32_type: IntType<'ctx>) -> Result<IntValue<'ctx>, String> {
    match resolve(v, temps, locals, i32_type) {
        RtValue::Int(i) => Ok(i),
        RtValue::Struct(_) => Err("expected an integer value, found a struct".to_string()),
    }
}

#[allow(clippy::too_many_arguments)]
fn compile_instructions<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    program: &IRProgram,
    function: FunctionValue<'ctx>,
    instructions: &[Instruction],
    temps: &mut [Option<RtValue<'ctx>>],
    locals: &mut [Option<RtValue<'ctx>>],
    i32_type: IntType<'ctx>,
) -> Result<bool, String> {
    for instr in instructions {
        match instr {
            Instruction::Add(dest, l, r) => {
                let result = builder
                    .build_int_add(resolve_int(*l, temps, locals, i32_type)?, resolve_int(*r, temps, locals, i32_type)?, "addtmp")
                    .map_err(|e| format!("codegen error: {e}"))?;
                temps[*dest] = Some(RtValue::Int(result));
            }
            Instruction::Sub(dest, l, r) => {
                let result = builder
                    .build_int_sub(resolve_int(*l, temps, locals, i32_type)?, resolve_int(*r, temps, locals, i32_type)?, "subtmp")
                    .map_err(|e| format!("codegen error: {e}"))?;
                temps[*dest] = Some(RtValue::Int(result));
            }
            Instruction::Mul(dest, l, r) => {
                let result = builder
                    .build_int_mul(resolve_int(*l, temps, locals, i32_type)?, resolve_int(*r, temps, locals, i32_type)?, "multmp")
                    .map_err(|e| format!("codegen error: {e}"))?;
                temps[*dest] = Some(RtValue::Int(result));
            }
            Instruction::Div(dest, l, r) => {
                let result = builder
                    .build_int_signed_div(resolve_int(*l, temps, locals, i32_type)?, resolve_int(*r, temps, locals, i32_type)?, "divtmp")
                    .map_err(|e| format!("codegen error: {e}"))?;
                temps[*dest] = Some(RtValue::Int(result));
            }
            Instruction::VarDecl(slot, v) => {
                let value = resolve(*v, temps, locals, i32_type);
                locals[*slot] = Some(value);
            }
            Instruction::Call(dest, func_index) => {
                let callee_name = &program.functions[*func_index].name;
                let callee = module.get_function(callee_name).ok_or(format!("function '{callee_name}' not found"))?;
                let call_site_value = builder
                    .build_call(callee, &[], "calltmp")
                    .map_err(|e| format!("codegen error: {e}"))?;
                let call_result = call_site_value
                    .try_as_basic_value()
                    .left()
                    .ok_or("expected call to return a value")?
                    .into_int_value();
                temps[*dest] = Some(RtValue::Int(call_result));
            }
            Instruction::MakeStruct { dest, name, fields } => {
                let struct_type = struct_type_for(context, name, i32_type)
                    .ok_or_else(|| format!("unknown struct type '{name}'"))?;
                let mut aggregate = struct_type.get_undef();
                for (index, field) in fields.iter().enumerate() {
                    let field_value = resolve_int(*field, temps, locals, i32_type)?;
                    aggregate = builder
                        .build_insert_value(aggregate, field_value, index as u32, "structinit")
                        .map_err(|e| format!("codegen error: {e}"))?
                        .into_struct_value();
                }
                temps[*dest] = Some(RtValue::Struct(aggregate));
            }
            Instruction::Return(v) => {
                let value = resolve_int(*v, temps, locals, i32_type)?;
                builder
                    .build_return(Some(&value))
                    .map_err(|e| format!("codegen error: {e}"))?;
                return Ok(true);
            }
            Instruction::If { condition, then_branch, else_branch } => {
                let cond_value = resolve_int(*condition, temps, locals, i32_type)?;
                let zero = i32_type.const_int(0, false);
                let cond_bool = builder
                    .build_int_compare(IntPredicate::NE, cond_value, zero, "ifcond")
                    .map_err(|e| format!("codegen error: {e}"))?;

                let then_bb = context.append_basic_block(function, "then");
                let else_bb = context.append_basic_block(function, "else");
                let merge_bb = context.append_basic_block(function, "ifcont");

                builder
                    .build_conditional_branch(cond_bool, then_bb, else_bb)
                    .map_err(|e| format!("codegen error: {e}"))?;

                builder.position_at_end(then_bb);
                let then_terminated =
                    compile_instructions(context, module, builder, program, function, then_branch, temps, locals, i32_type)?;
                if !then_terminated {
                    builder.build_unconditional_branch(merge_bb).map_err(|e| format!("codegen error: {e}"))?;
                }

                builder.position_at_end(else_bb);
                let else_terminated = match else_branch {
                    Some(else_instructions) => {
                        compile_instructions(context, module, builder, program, function, else_instructions, temps, locals, i32_type)?
                    }
                    None => false,
                };
                if !else_terminated {
                    builder.build_unconditional_branch(merge_bb).map_err(|e| format!("codegen error: {e}"))?;
                }

                if then_terminated && else_terminated {
                    builder.position_at_end(merge_bb);
                    builder.build_unreachable().map_err(|e| format!("codegen error: {e}"))?;
                    return Ok(true);
                }

                builder.position_at_end(merge_bb);
            }
            Instruction::Assign(slot, v) => {
                let value = resolve(*v, temps, locals, i32_type);
                locals[*slot] = Some(value);
            }
        }
    }
    Ok(false)
}

pub fn compile_to_executable(program: &IRProgram, output_path: &Path) -> Result<(), String> {
    let context = Context::create();
    let module = context.create_module("vita_module");
    let builder = context.create_builder();

    let i32_type = context.i32_type();

    for func in &program.functions {
        let fn_type = i32_type.fn_type(&[], false);
        module.add_function(&func.name, fn_type, None);
    }

    for func in &program.functions {
        let function = module.get_function(&func.name).expect("function was just declared above");
        let entry_block = context.append_basic_block(function, "entry");
        builder.position_at_end(entry_block);

        let mut temps: Vec<Option<RtValue>> = vec![None; func.temp_count];
        let mut locals: Vec<Option<RtValue>> = vec![None; func.local_count];

        compile_instructions(&context, &module, &builder, program, function, &func.instructions, &mut temps, &mut locals, i32_type)?;
    }

    if let Err(e) = module.verify() {
        return Err(format!("generated invalid LLVM IR:\n{e}"));
    }

    Target::initialize_native(&InitializationConfig::default())
        .map_err(|e| format!("couldn't initialize LLVM target: {e}"))?;

    let triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&triple).map_err(|e| format!("unknown target: {e}"))?;
    let target_machine = target
        .create_target_machine(
            &triple,
            &TargetMachine::get_host_cpu_name().to_string(),
            &TargetMachine::get_host_cpu_features().to_string(),
            OptimizationLevel::Default,
            RelocMode::Default,
            CodeModel::Default,
        )
        .ok_or("couldn't create a target machine for this CPU")?;

    module
        .run_passes("default<O2>", &target_machine, PassBuilderOptions::create())
        .map_err(|e| format!("optimization failed: {e}"))?;

    let ll_path = output_path.with_extension("ll");
    module
        .print_to_file(&ll_path)
        .map_err(|e| format!("couldn't write .ll file: {e}"))?;

    let obj_path = output_path.with_extension("o");
    let asm_path = output_path.with_extension("s");
    target_machine
        .write_to_file(&module, FileType::Assembly, &asm_path)
        .map_err(|e| format!("couldn't write assembly file: {e}"))?;
    target_machine
        .write_to_file(&module, FileType::Object, &obj_path)
        .map_err(|e| format!("couldn't write object file: {e}"))?;

    let output = Command::new("cc")
        .arg(&obj_path)
        .arg("-o")
        .arg(output_path)
        .output()
        .map_err(|e| format!("couldn't run the system linker ('cc'): {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("linking failed:\n{stderr}"));
    }

    Ok(())
}
