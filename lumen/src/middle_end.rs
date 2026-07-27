use crate::parser::{BinaryOp, Expr, Stmt};
use crate::typechecker::TypedProgram;

#[derive(Clone, Copy)]
pub enum Value {
    Const(i64),
    Temp(usize),
}

pub enum Instr {
    Add(usize, Value, Value),
    Sub(usize, Value, Value),
    Mul(usize, Value, Value),
    Div(usize, Value, Value),
    VarDecl(usize, Value),
    Return(Value),
}

pub fn lower(program: TypedProgram) -> Vec<Instr> {
    let mut instructions = vec![];
    let mut next_temp = 0;

    for stmt in program.body {
        match stmt {
            Stmt::Return(expr) => {
                let value = lower_expr(expr, &mut instructions, &mut next_temp);
                instructions.push(Instr::Return(value));
            }
            Stmt::VarDecl(_, expr) => {
                let value = lower_expr(expr, &mut instructions, &mut next_temp);
                instructions.push(Instr::VarDecl(next_temp, value));
                next_temp += 1;
            }
            Stmt::ValDecl(_, expr) => {
                let value = lower_expr(expr, &mut instructions, &mut next_temp);
                instructions.push(Instr::VarDecl(next_temp, value));
                next_temp += 1;
            }
        }
    }

    instructions
}

fn lower_expr(expr: Expr, instructions: &mut Vec<Instr>, next_temp: &mut usize) -> Value {
    match expr {
        Expr::Int(n) => Value::Const(n),
        Expr::Binary { op, left, right } => {
            let left_val = lower_expr(*left, instructions, next_temp);
            let right_val = lower_expr(*right, instructions, next_temp);

            let dest = *next_temp;
            *next_temp += 1;

            instructions.push(match op {
                BinaryOp::Plus => Instr::Add(dest, left_val, right_val),
                BinaryOp::Minus => Instr::Sub(dest, left_val, right_val),
                BinaryOp::Star => Instr::Mul(dest, left_val, right_val),
                BinaryOp::Slash => Instr::Div(dest, left_val, right_val),
            });

            Value::Temp(dest)
        }
        Expr::Var(_) => {
            Value::Temp(0)
        }
    }
}