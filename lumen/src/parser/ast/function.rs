use crate::parser::Stmt;

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub body: Vec<Stmt>,
    pub return_type: Option<String>,
}