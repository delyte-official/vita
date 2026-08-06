use std::fmt;

use crate::parser::Function;

use crate::parser::Field;
use super::PrimitiveConstructor;

pub enum PrimitiveClassItem {
    Field(Field),
    Constructor(PrimitiveConstructor),
    Method(Function),
}

pub struct PrimitiveClass {
    pub name: String,
    pub items: Vec<PrimitiveClassItem>,
}

impl fmt::Debug for PrimitiveClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "primitive class {} {{\n", self.name)?;
        for item in &self.items {
            match item {
                PrimitiveClassItem::Field(field) => write!(f, "\t{:#?}\n", field)?,
                PrimitiveClassItem::Constructor(construct) => write!(f, "\t{:#?}\n", construct)?,
                PrimitiveClassItem::Method(method) => method.fmt_with_indent(f, 1)?,
            }
            writeln!(f)?;
        }
        write!(f,"}}")
    }
}