use crate::{parser::BinaryOp, type_checker::ExprType};

#[derive(Debug)]
pub enum TypedExpr {
    Int(i64),
    Binary {
        op: BinaryOp,
        left: Box<TypedExpr>,
        right: Box<TypedExpr>,
        ty: ExprType,
    },
    Var(usize),
}

impl TypedExpr {
    pub fn ty(&self) -> ExprType {
        match self {
            TypedExpr::Int(_) => ExprType::I32,
            TypedExpr::Binary { ty, .. } => *ty,
            TypedExpr::Var(_) => ExprType::I32,
        }
    }
}