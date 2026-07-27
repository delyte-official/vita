use super::BinaryOp;

#[derive(Debug)]
pub enum Expr {
    Int(i64),
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Var(String),
}

impl Expr {
    pub fn binary(op: BinaryOp, left: Expr, right: Expr) -> Expr {
        Expr::Binary { op, left: Box::new(left), right: Box::new(right) }
    }
}