use crate::binder::BoundProgram;
use crate::parser::{Expr, Stmt};

#[derive(Debug, Clone, Copy)]
pub enum Type {
    I32,
}

pub struct TypedProgram {
    pub return_type: Type,
    pub body: Vec<Stmt>,
}

pub fn check(program: BoundProgram) -> Result<TypedProgram, String> {
    let return_type = Type::I32;

    for stmt in &program.body {
        let Stmt::Return(expr) = stmt;
        let found_type = infer(expr);
        if !matches!((return_type, found_type), (Type::I32, Type::I32)) {
            return Err("return type doesn't match main's declared return type".to_string());
        }
    }

    Ok(TypedProgram {
        return_type,
        body: program.body,
    })
}

fn infer(expr: &Expr) -> Type {
    match expr {
        Expr::Int(_) => Type::I32,
    }
}
