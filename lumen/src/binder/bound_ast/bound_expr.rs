use std::fmt;

use crate::parser::BinaryOp;

#[derive(Debug)]
pub enum BoundedTemplatePart {
    Text(String),
    Expr(Box<BoundedExpr>),
}

pub enum BoundedExpr {
    Literal(String),
    TemplateLiteral(Vec<BoundedTemplatePart>),
    Binary {
        op: BinaryOp,
        left: Box<BoundedExpr>,
        right: Box<BoundedExpr>,
    },
    Var(usize, String),
    FuncCall(usize, String),
}

impl fmt::Debug for BoundedExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BoundedExpr::Literal(s) => write!(f, "{}", s),
            BoundedExpr::TemplateLiteral(parts) => {
                let parts_str = parts
                    .iter()
                    .map(|part| match part {
                        BoundedTemplatePart::Text(s) => format!("{}", s),
                        BoundedTemplatePart::Expr(e) => format!("{{{:#?}}}", e),
                    })
                    .collect::<Vec<String>>()
                    .join("");
                write!(f, "{}", parts_str)
            }
            BoundedExpr::Binary { op, left, right } => {
                write!(f, "{:#?} {:#?} {:#?}", left, op, right)
            }
            BoundedExpr::Var(index, name) => write!(f, "{}#{}", name, index),
            BoundedExpr::FuncCall(index, name) => write!(f, "{}#{}()", name, index),
        }
    }
}
