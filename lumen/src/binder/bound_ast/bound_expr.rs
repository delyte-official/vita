use crate::parser::BinaryOp;

#[derive(Debug)]
pub enum BoundedExpr {
    Int(i64),
    Binary {
        op: BinaryOp,
        left: Box<BoundedExpr>,
        right: Box<BoundedExpr>,
    },
    Var(usize),
    FuncCall(usize),
}