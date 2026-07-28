use crate::binder::BoundedExpr;

#[derive(Debug)]
pub enum BoundedStmt {
    Return(BoundedExpr),
    VarDecl(usize, BoundedExpr),
    ValDecl(usize, BoundedExpr),
}