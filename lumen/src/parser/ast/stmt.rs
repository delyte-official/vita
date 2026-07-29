use super::Expr;

#[derive(Debug)]
pub enum Stmt {
    Return(Expr),
    VarDecl(String, Expr),
    ValDecl(String, Expr),
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        elif_branches: Vec<(Expr, Vec<Stmt>)>,
        else_branch: Option<Vec<Stmt>>,
    },
}
