use std::fmt;

use super::{Function, PrimitiveClass};

pub enum Definition {
    Function(Function),
    PrimitiveClass(PrimitiveClass),
}

pub struct Program {
    pub definitions: Vec<Definition>,
}

impl fmt::Debug for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = self
            .definitions
            .iter()
            .map(|def| match def {
                Definition::Function(func) => format!("{:#?}", func),
                Definition::PrimitiveClass(class) => format!("{:#?}", class),
            })
            .collect::<Vec<String>>()
            .join("\n\n");
        write!(f, "{}\n", str)
    }
}