use std::fmt;

use crate::parser::Stmt;

pub struct Function {
    pub name: String,
    pub body: Vec<Stmt>,
    pub return_type: Option<String>,
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "func {}(){} {{\n", self.name, match &self.return_type {
            Some(t) => format!(": {}", t),
            None => String::new(),
        })?;
        let mut index = 0;
        for stmt in &self.body {
            if index > 0 {
                writeln!(f)?;
            }
            stmt.fmt_with_indent(f, 1)?;
            index += 1;
        }
        write!(f, "\n}}")
    }
}