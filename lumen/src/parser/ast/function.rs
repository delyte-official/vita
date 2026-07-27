use super::Stmt;

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub body: Vec<Stmt>,
}