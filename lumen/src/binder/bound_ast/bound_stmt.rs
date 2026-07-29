use crate::binder::BoundedExpr;

#[derive(Debug)]
pub enum BoundedStmt {
    Return(BoundedExpr),
    VarDecl(usize, BoundedExpr),
    ValDecl(usize, BoundedExpr),
    If {
        condition: BoundedExpr,
        then_branch: Vec<BoundedStmt>,
        elif_branches: Option<Vec<(BoundedExpr, Vec<BoundedStmt>)>>,
        else_branch: Option<Vec<BoundedStmt>>,
    },
    Assign(usize, BoundedExpr),
    FuncCall(usize),
}
