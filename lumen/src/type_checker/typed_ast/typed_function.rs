use super::TypedStmt;
use crate::type_checker::ExprType;

#[derive(Debug)]
pub struct TypedFunction {
    pub name: String,
    pub body: Vec<TypedStmt>,
    pub local_count: usize,
    pub return_type: ExprType,
}
