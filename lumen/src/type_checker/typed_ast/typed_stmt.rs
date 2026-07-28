use crate::type_checker::TypedExpr;

#[derive(Debug)]
pub enum TypedStmt {
    Return(TypedExpr),
    VarDecl(usize, TypedExpr),
    ValDecl(usize, TypedExpr),
}