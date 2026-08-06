use std::fmt;

use super::BinaryOp;

pub enum LiteralPart {
    Text(String),
    Expr(Expr),
}

pub enum Expr {
    Literal(String),
    TemplateLiteral(Vec<LiteralPart>),
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Parenthesized(Box<Expr>),
    Var(String),
    FuncCall { name: String },
}

impl Expr {
    pub fn binary(op: BinaryOp, left: Expr, right: Expr) -> Expr {
        Expr::Binary { op, left: Box::new(left), right: Box::new(right) }
    }
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Literal(s) => write!(f, "{}", s),
            Expr::TemplateLiteral(parts) => {
                let parts_str = parts
                    .iter()
                    .map(|part| match part {
                        LiteralPart::Text(s) => format!("{}", s),
                        LiteralPart::Expr(e) => format!("{{{:#?}}}", e),
                    })
                    .collect::<Vec<String>>()
                    .join("");
                write!(f, "{}", parts_str)
            }
            Expr::Binary { op, left, right } => {
                write!(f, "{:#?} {:#?} {:#?}", left, op, right)
            }
            Expr::Parenthesized(expr) => write!(f, "({:#?})", expr),
            Expr::Var(name) => write!(f, "{}", name),
            Expr::FuncCall { name } => write!(f, "{}()", name),
        }
    }
}
