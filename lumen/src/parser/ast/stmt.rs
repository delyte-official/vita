use super::Expr;

#[derive(Debug)]
pub enum Stmt {
    Return(Expr),
    VarDecl(String, Expr),
    ValDecl(String, Expr),
}