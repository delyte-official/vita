use super::Expr;

#[derive(Debug)]
pub enum Stmt {
    Return(Expr),
}