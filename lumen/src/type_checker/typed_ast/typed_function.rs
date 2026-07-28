use super::TypedStmt;

#[derive(Debug)]
pub struct TypedFunction {
    pub name: String,
    pub body: Vec<TypedStmt>,
    pub local_count: usize,
}
