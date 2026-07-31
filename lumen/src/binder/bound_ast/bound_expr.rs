use crate::parser::BinaryOp;

#[derive(Debug)]
pub enum BoundedExpr {
    Literal(String),
    UnresolvedName(String),
    Binary {
        op: BinaryOp,
        left: Box<BoundedExpr>,
        right: Box<BoundedExpr>,
    },
    Var(usize),
    FuncCall(usize),
}