use std::fmt;

use crate::{parser::BinaryOp, type_checker::ExprType};

#[derive(Debug)]
pub enum TypedTemplatePart {
    Text(String),
    Expr(TypedExpr),
}

pub enum TypedExpr {
    Literal {
        value: String,
        ty: ExprType,
    },
    Template(Vec<TypedTemplatePart>),
    Binary {
        op: BinaryOp,
        left: Box<TypedExpr>,
        right: Box<TypedExpr>,
        ty: ExprType,
    },
    Var(usize, String),
    FuncCall(usize, String),
}

impl TypedExpr {
    pub fn ty(&self) -> ExprType {
        match self {
            TypedExpr::Literal { ty, .. } => *ty,
            TypedExpr::Template(_) => ExprType::String,
            TypedExpr::Binary { ty, .. } => *ty,
            TypedExpr::Var(..) => ExprType::I32,
            TypedExpr::FuncCall(..) => ExprType::I32,
        }
    }
}

impl fmt::Debug for TypedExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypedExpr::Literal{value, ..} => write!(f, "{}", value),
            TypedExpr::Template(parts) => {
                let parts_str = parts
                    .iter()
                    .map(|part| match part {
                        TypedTemplatePart::Text(s) => format!("{}", s),
                        TypedTemplatePart::Expr(e) => format!("{{{:#?}}}", e),
                    })
                    .collect::<Vec<String>>()
                    .join("");
                write!(f, "{}", parts_str)
            }
            TypedExpr::Binary { op, left, right, .. } => {
                write!(f, "{:#?} {:#?} {:#?}", left, op, right)
            }
            TypedExpr::Var(index, name) => write!(f, "{}#{}", name, index),
            TypedExpr::FuncCall(index, name) => write!(f, "{}#{}()", name, index),
        }
    }
}