use std::fmt;

use crate::type_checker::TypedFunction;

pub struct TypedProgram {
    pub functions: Vec<TypedFunction>,
}

impl fmt::Debug for TypedProgram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = self
            .functions
            .iter()
            .map(|f| format!("{:#?}", f))
            .collect::<Vec<String>>()
            .join("\n\n");
        write!(f, "{}\n", str)
    }
}