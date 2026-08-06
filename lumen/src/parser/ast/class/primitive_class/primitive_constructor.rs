use std::fmt;

use crate::parser::{LiteralRepresentation, Stmt};

pub struct PrimitiveConstructor {
    pub literal_representation: LiteralRepresentation,
    pub body: Vec<Stmt>,
}

impl fmt::Debug for PrimitiveConstructor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "constructor({:#?}) {{\n", self.literal_representation)?;
        for stmt in &self.body {
            write!(f, "\t\t{:#?}\n", stmt)?;
        }
        write!(f,"\t}}")
    }
}