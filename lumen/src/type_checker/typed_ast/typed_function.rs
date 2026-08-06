use std::fmt;

use super::TypedStmt;
use crate::type_checker::ExprType;

pub struct TypedFunction {
    pub index: usize,
    pub name: String,
    pub body: Vec<TypedStmt>,
    pub local_count: usize,
    pub return_type: ExprType,
}

impl fmt::Debug for TypedFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "func {}#{}(): {:#?} {{\n", self.name, self.index, self.return_type)?;
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