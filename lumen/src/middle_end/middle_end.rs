use crate::middle_end::*;
use crate::parser::BinaryOp;
use crate::type_checker::*;

pub fn lower(program: TypedProgram) -> IRProgram {
    let mut ir_functions = vec![];

    for func in program.functions {
        let mut instructions: Vec<Instruction> = vec![];
        let mut next_temp = 0;
        for stmt in func.body {
            match stmt {
                TypedStmt::Return(expr) => {
                    let value = lower_expr(expr, &mut instructions, &mut next_temp);
                    instructions.push(Instruction::Return(value));
                }
                TypedStmt::VarDecl(slot, expr) | TypedStmt::ValDecl(slot, expr) => {
                    let value = lower_expr(expr, &mut instructions, &mut next_temp);
                    instructions.push(Instruction::VarDecl(slot, value));
                }
            }
        }
        ir_functions.push(IRFunction {
            name: func.name,
            instructions,
            local_count: func.local_count,
            temp_count: next_temp,
        });
    }

    IRProgram {
        functions: ir_functions,
    }
}

fn lower_expr(expr: TypedExpr, instructions: &mut Vec<Instruction>, next_temp: &mut usize) -> Value {
    match expr {
        TypedExpr::Int(n) => Value::Const(n),
        TypedExpr::Var(slot) => Value::Var(slot),
        TypedExpr::Binary { op, left, right, .. } => {
            let left_val = lower_expr(*left, instructions, next_temp);
            let right_val = lower_expr(*right, instructions, next_temp);
            let dest = *next_temp;
            *next_temp += 1;
            instructions.push(match op {
                BinaryOp::Plus => Instruction::Add(dest, left_val, right_val),
                BinaryOp::Minus => Instruction::Sub(dest, left_val, right_val),
                BinaryOp::Star => Instruction::Mul(dest, left_val, right_val),
                BinaryOp::Slash => Instruction::Div(dest, left_val, right_val),
            });
            Value::Temp(dest)
        }
    }
}