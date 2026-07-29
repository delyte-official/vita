use crate::binder::BoundedExpr;

#[derive(Debug)]
pub enum BoundedStmt {
    Return(BoundedExpr),
    VarDecl(usize, BoundedExpr),
    ValDecl(usize, BoundedExpr),
    If {
        condition: BoundedExpr,
        then_branch: Vec<BoundedStmt>,
        elif_branches: Vec<(BoundedExpr, Vec<BoundedStmt>)>,
        else_branch: Option<Vec<BoundedStmt>>,
    },
}
