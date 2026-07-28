use crate::binder::BoundedStmt;

pub struct BoundedFunction {
    pub name: String,
    pub body: Vec<BoundedStmt>,
    pub local_count: usize,
}
