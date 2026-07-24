use std::path::Path;
use std::process::Command;

use inkwell::context::Context;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::values::IntValue;
use inkwell::OptimizationLevel;

use crate::middle_end::{Instr, Value};

fn resolve<'ctx>(v: Value, temps: &[IntValue<'ctx>], i32_type: inkwell::types::IntType<'ctx>) -> IntValue<'ctx> {
    match v {
        Value::Const(n) => i32_type.const_int(n as u64, true),
        Value::Temp(i) => temps[i],
    }
}

pub fn compile_to_executable(instructions: &[Instr], output_path: &Path) -> Result<(), String> {
    let context = Context::create();
    let module = context.create_module("vita_module");
    let builder = context.create_builder();

    let i32_type = context.i32_type();
    let fn_type = i32_type.fn_type(&[], false);
    let function = module.add_function("main", fn_type, None);
    let entry_block = context.append_basic_block(function, "entry");
    builder.position_at_end(entry_block);

    let mut temps: Vec<IntValue> = vec![];

    for instr in instructions {
        match instr {
            Instr::Add(_, l, r) => {
                let result = builder
                    .build_int_add(resolve(*l, &temps, i32_type), resolve(*r, &temps, i32_type), "addtmp")
                    .map_err(|e| format!("codegen error: {e}"))?;
                temps.push(result);
            }
            Instr::Sub(_, l, r) => {
                let result = builder
                    .build_int_sub(resolve(*l, &temps, i32_type), resolve(*r, &temps, i32_type), "subtmp")
                    .map_err(|e| format!("codegen error: {e}"))?;
                temps.push(result);
            }
            Instr::Mul(_, l, r) => {
                let result = builder
                    .build_int_mul(resolve(*l, &temps, i32_type), resolve(*r, &temps, i32_type), "multmp")
                    .map_err(|e| format!("codegen error: {e}"))?;
                temps.push(result);
            }
            Instr::Div(_, l, r) => {
                let result = builder
                    .build_int_signed_div(resolve(*l, &temps, i32_type), resolve(*r, &temps, i32_type), "divtmp")
                    .map_err(|e| format!("codegen error: {e}"))?;
                temps.push(result);
            }
            Instr::Return(v) => {
                let value = resolve(*v, &temps, i32_type);
                builder
                    .build_return(Some(&value))
                    .map_err(|e| format!("codegen error: {e}"))?;
            }
        }
    }

    if let Err(e) = module.verify() {
        return Err(format!("generated invalid LLVM IR:\n{e}"));
    }

    let ll_path = output_path.with_extension("ll");
    module
        .print_to_file(&ll_path)
        .map_err(|e| format!("couldn't write .ll file: {e}"))?;

    Target::initialize_native(&InitializationConfig::default())
        .map_err(|e| format!("couldn't initialize LLVM target: {e}"))?;

    let triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&triple).map_err(|e| format!("unknown target: {e}"))?;
    let target_machine = target
        .create_target_machine(
            &triple,
            &TargetMachine::get_host_cpu_name().to_string(),
            &TargetMachine::get_host_cpu_features().to_string(),
            OptimizationLevel::None,
            RelocMode::Default,
            CodeModel::Default,
        )
        .ok_or("couldn't create a target machine for this CPU")?;

    let obj_path = output_path.with_extension("o");
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