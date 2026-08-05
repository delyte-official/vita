use std::fmt;

use crate::binder::BoundedFunction;

pub struct BoundedProgram {
    pub functions: Vec<BoundedFunction>,
}


impl fmt::Debug for BoundedProgram {
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