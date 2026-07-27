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
        match stmt {
            Stmt::Return(expr) => {
                let found_type = infer(expr);
                if !matches!((return_type, found_type), (Type::I32, Type::I32)) {
                    return Err("return type doesn't match main's declared return type".to_string());
                }
            }
            Stmt::VarDecl(_, expr) | Stmt::ValDecl(_, expr) => {
                let found_type = infer(expr);
                if !matches!((return_type, found_type), (Type::I32, Type::I32)) {
                    return Err("variable declaration type doesn't match main's declared return type".to_string());
                }
            }
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
        Expr::Binary { left, right, .. } => {
            let left_type = infer(left);
            let right_type = infer(right);
            if !matches!((left_type, right_type), (Type::I32, Type::I32)) {
                panic!("binary operation operands must be of type i32");
            }
            Type::I32
        }
        Expr::Var(_) => Type::I32,
    }
}
