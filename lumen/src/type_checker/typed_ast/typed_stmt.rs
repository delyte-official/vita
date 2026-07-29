use crate::type_checker::TypedExpr;

#[derive(Debug)]
pub enum TypedStmt {
    Return(TypedExpr),
    VarDecl(usize, TypedExpr),
    ValDecl(usize, TypedExpr),
    If {
        condition: TypedExpr,
        then_branch: Vec<TypedStmt>,
        elif_branches: Option<Vec<(TypedExpr, Vec<TypedStmt>)>>,
        else_branch: Option<Vec<TypedStmt>>,
    },
}
