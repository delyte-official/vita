use super::Expr;

#[derive(Debug)]
pub enum Stmt {
    Return(Expr),
    VarDecl(String, Option<String>, Expr),
    ValDecl(String, Option<String>, Expr),
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        elif_branches: Option<Vec<(Expr, Vec<Stmt>)>>,
        else_branch: Option<Vec<Stmt>>,
    },
    Assign(String, Expr),
    FuncCall{ name: String },
}
