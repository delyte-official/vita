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
use inkwell::values::{FunctionValue, IntValue, PointerValue, StructValue};
use inkwell::{AddressSpace, IntPredicate, OptimizationLevel};

use crate::middle_end::{IRProgram, Instruction, Value};
use crate::type_checker::ExprType;

#[derive(Clone, Copy)]
enum RtValue<'ctx> {
    Int(IntValue<'ctx>),
    Struct(StructValue<'ctx>),
    Str(PointerValue<'ctx>),
}

fn struct_type_for<'ctx>(context: &'ctx Context, field_count: usize, i32_type: IntType<'ctx>) -> StructType<'ctx> {
    let field_types = vec![i32_type.into(); field_count];
    context.struct_type(&field_types, false)
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
        RtValue::Str(_) => Err("expected an integer value, found a string".to_string()),
    }
}

fn resolve_str<'ctx>(v: Value, temps: &[Option<RtValue<'ctx>>], locals: &[Option<RtValue<'ctx>>], i32_type: IntType<'ctx>) -> Result<PointerValue<'ctx>, String> {
    match resolve(v, temps, locals, i32_type) {
        RtValue::Str(p) => Ok(p),
        RtValue::Int(_) => Err("expected a string value, found an integer".to_string()),
        RtValue::Struct(_) => Err("expected a string value, found a struct".to_string()),
    }
}

fn get_extern_fn<'ctx>(module: &Module<'ctx>, name: &str) -> FunctionValue<'ctx> {
    module.get_function(name).unwrap_or_else(|| panic!("extern function '{name}' was not declared"))
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
    let i64_type = context.i64_type();
    let i8_type = context.i8_type();

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
            Instruction::MakeStruct { dest, name: _, fields } => {
                let struct_type = struct_type_for(context, fields.len(), i32_type);
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
            Instruction::MakeString(dest, content) => {
                let global_str = builder
                    .build_global_string_ptr(content, "strlit")
                    .map_err(|e| format!("codegen error: {e}"))?;
                temps[*dest] = Some(RtValue::Str(global_str.as_pointer_value()));
            }
            Instruction::IntToString(dest, v) => {
                let int_val = resolve_int(*v, temps, locals, i32_type)?;
                let malloc_fn = get_extern_fn(module, "malloc");
                let snprintf_fn = get_extern_fn(module, "snprintf");
                let buf_size = i64_type.const_int(12, false);
                let buf = builder
                    .build_call(malloc_fn, &[buf_size.into()], "intbuf")
                    .map_err(|e| format!("codegen error: {e}"))?
                    .try_as_basic_value()
                    .left()
                    .ok_or("expected malloc to return a value")?
                    .into_pointer_value();
                let fmt = builder
                    .build_global_string_ptr("%d", "int_fmt")
                    .map_err(|e| format!("codegen error: {e}"))?
                    .as_pointer_value();
                builder
                    .build_call(snprintf_fn, &[buf.into(), buf_size.into(), fmt.into(), int_val.into()], "snprintf_call")
                    .map_err(|e| format!("codegen error: {e}"))?;
                temps[*dest] = Some(RtValue::Str(buf));
            }
            Instruction::BoolToString(dest, v) => {
                let int_val = resolve_int(*v, temps, locals, i32_type)?;
                let zero = i32_type.const_int(0, false);
                let is_true = builder
                    .build_int_compare(IntPredicate::NE, int_val, zero, "booltruthy")
                    .map_err(|e| format!("codegen error: {e}"))?;
                let true_str = builder
                    .build_global_string_ptr("true", "true_str")
                    .map_err(|e| format!("codegen error: {e}"))?
                    .as_pointer_value();
                let false_str = builder
                    .build_global_string_ptr("false", "false_str")
                    .map_err(|e| format!("codegen error: {e}"))?
                    .as_pointer_value();
                let selected = builder
                    .build_select(is_true, true_str, false_str, "boolstr")
                    .map_err(|e| format!("codegen error: {e}"))?
                    .into_pointer_value();
                temps[*dest] = Some(RtValue::Str(selected));
            }
            Instruction::CharToString(dest, v) => {
                let int_val = resolve_int(*v, temps, locals, i32_type)?;
                let char_byte = builder
                    .build_int_truncate(int_val, i8_type, "charbyte")
                    .map_err(|e| format!("codegen error: {e}"))?;
                let malloc_fn = get_extern_fn(module, "malloc");
                let buf_size = i64_type.const_int(2, false);
                let buf = builder
                    .build_call(malloc_fn, &[buf_size.into()], "charbuf")
                    .map_err(|e| format!("codegen error: {e}"))?
                    .try_as_basic_value()
                    .left()
                    .ok_or("expected malloc to return a value")?
                    .into_pointer_value();
                builder.build_store(buf, char_byte).map_err(|e| format!("codegen error: {e}"))?;
                let second_byte_ptr = unsafe {
                    builder.build_gep(i8_type, buf, &[i64_type.const_int(1, false)], "charbuf_null")
                }
                .map_err(|e| format!("codegen error: {e}"))?;
                builder
                    .build_store(second_byte_ptr, i8_type.const_zero())
                    .map_err(|e| format!("codegen error: {e}"))?;
                temps[*dest] = Some(RtValue::Str(buf));
            }
            Instruction::Concat(dest, l, r) => {
                let left_str = resolve_str(*l, temps, locals, i32_type)?;
                let right_str = resolve_str(*r, temps, locals, i32_type)?;
                let strlen_fn = get_extern_fn(module, "strlen");
                let malloc_fn = get_extern_fn(module, "malloc");
                let strcpy_fn = get_extern_fn(module, "strcpy");
                let strcat_fn = get_extern_fn(module, "strcat");

                let left_len = builder
                    .build_call(strlen_fn, &[left_str.into()], "leftlen")
                    .map_err(|e| format!("codegen error: {e}"))?
                    .try_as_basic_value()
                    .left()
                    .ok_or("expected strlen to return a value")?
                    .into_int_value();
                let right_len = builder
                    .build_call(strlen_fn, &[right_str.into()], "rightlen")
                    .map_err(|e| format!("codegen error: {e}"))?
                    .try_as_basic_value()
                    .left()
                    .ok_or("expected strlen to return a value")?
                    .into_int_value();
                let total = builder
                    .build_int_add(left_len, right_len, "totallen")
                    .map_err(|e| format!("codegen error: {e}"))?;
                let total_plus_one = builder
                    .build_int_add(total, i64_type.const_int(1, false), "totalpad")
                    .map_err(|e| format!("codegen error: {e}"))?;
                let buf = builder
                    .build_call(malloc_fn, &[total_plus_one.into()], "concatbuf")
                    .map_err(|e| format!("codegen error: {e}"))?
                    .try_as_basic_value()
                    .left()
                    .ok_or("expected malloc to return a value")?
                    .into_pointer_value();
                builder
                    .build_call(strcpy_fn, &[buf.into(), left_str.into()], "strcpy_call")
                    .map_err(|e| format!("codegen error: {e}"))?;
                builder
                    .build_call(strcat_fn, &[buf.into(), right_str.into()], "strcat_call")
                    .map_err(|e| format!("codegen error: {e}"))?;
                temps[*dest] = Some(RtValue::Str(buf));
            }
            Instruction::Return(v) => {
                match resolve(*v, temps, locals, i32_type) {
                    RtValue::Int(i) => builder.build_return(Some(&i)),
                    RtValue::Struct(s) => builder.build_return(Some(&s)),
                    RtValue::Str(p) => builder.build_return(Some(&p)),
                }
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
    let i64_type = context.i64_type();
    let ptr_type = context.ptr_type(AddressSpace::default());

    module.add_function("malloc", ptr_type.fn_type(&[i64_type.into()], false), None);
    module.add_function("strlen", i64_type.fn_type(&[ptr_type.into()], false), None);
    module.add_function("strcpy", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), None);
    module.add_function("strcat", ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false), None);
    module.add_function("snprintf", i32_type.fn_type(&[ptr_type.into(), i64_type.into(), ptr_type.into()], true), None);

    for func in &program.functions {
        let fn_type = match func.return_type {
            ExprType::Void => context.void_type().fn_type(&[], false),
            // Bools and chars are both represented as plain i32 values at
            // runtime (see lower_expr in the middle end), so they share the
            // i32 function signature.
            ExprType::I32 | ExprType::Bool | ExprType::Char => i32_type.fn_type(&[], false),
            ExprType::Str => ptr_type.fn_type(&[], false),
            ExprType::Struct(name) => {
                return Err(format!(
                    "function '{}' returns struct '{}', but returning a struct from a function isn't supported by codegen yet",
                    func.name, name
                ));
            }
        };
        module.add_function(&func.name, fn_type, None);
    }

    for func in &program.functions {
        let function = module.get_function(&func.name).expect("function was just declared above");
        let entry_block = context.append_basic_block(function, "entry");
        builder.position_at_end(entry_block);

        let mut temps: Vec<Option<RtValue>> = vec![None; func.temp_count];
        let mut locals: Vec<Option<RtValue>> = vec![None; func.local_count];

        let terminated = compile_instructions(&context, &module, &builder, program, function, &func.instructions, &mut temps, &mut locals, i32_type)?;

        if !terminated {
            if func.return_type == ExprType::Void {
                // A void function never contains an explicit `return` (the
                // type checker forbids it), so control always falls off the
                // end - that's exactly where the implicit `ret void` goes.
                builder.build_return(None).map_err(|e| format!("codegen error: {e}"))?;
            } else {
                return Err(format!(
                    "internal compiler error: function '{}' should return on all paths but its generated IR doesn't - this should have been caught by the type checker",
                    func.name
                ));
            }
        }
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
