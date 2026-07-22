use crate::parser::{Expr, Stmt};
use crate::typechecker::TypedProgram;

pub enum Instr {
    Return(i64),
}

pub fn lower(program: TypedProgram) -> Vec<Instr> {
    let mut instructions = vec![];

    for stmt in program.body {
        let Stmt::Return(expr) = stmt;
        instructions.push(lower_expr(expr));
    }

    instructions
}

fn lower_expr(expr: Expr) -> Instr {
    match expr {
        Expr::Int(n) => Instr::Return(n),
    }
}
