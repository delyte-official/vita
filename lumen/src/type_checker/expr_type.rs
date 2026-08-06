use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ExprType {
    Void,
    I32,
    Bool,
    String,
    Char,
}

impl ExprType {
    pub fn to_string(&self) -> String {
        match self {
            ExprType::Void => "void".to_string(),
            ExprType::I32 => "i32".to_string(),
            ExprType::Bool => "bool".to_string(),
            ExprType::String => "string".to_string(),
            ExprType::Char => "char".to_string(),
        }
    }

    pub fn from_str(s: &str) -> Option<ExprType> {
        match s {
            "void" => Some(ExprType::Void),
            "int" | "i32" => Some(ExprType::I32),
            "bool" => Some(ExprType::Bool),
            "string" => Some(ExprType::String),
            "char" => Some(ExprType::Char),
            _ => None,
        }
    }
}

impl fmt::Debug for ExprType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}#primitive", match self {
            ExprType::Void => "void",
            ExprType::I32 => "i32",
            ExprType::Bool => "bool",
            ExprType::String => "string",
            ExprType::Char => "char",
        })
    }
}