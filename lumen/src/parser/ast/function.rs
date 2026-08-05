use std::fmt;

use crate::parser::Stmt;

pub struct Function {
    pub name: String,
    pub body: Vec<Stmt>,
    pub return_type: Option<String>,
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let body_str = self
            .body
            .iter()
            .map(|stmt| format!("{:#?}", stmt))
            .collect::<Vec<String>>()
            .join("\n");
        write!(f, "func {}(){} {{\n{}\n}}", self.name, match &self.return_type {
            Some(t) => format!(": {}", t),
            None => String::new(),
        }, body_str)
    }
}