use std::fmt;

use super::Function;

pub struct Program {
    pub functions: Vec<Function>,
}

impl fmt::Debug for Program {
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