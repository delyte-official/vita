#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExprType {
    Void,
    I32,
    Bool,
    Str,
    Char,
    Struct(&'static str),
}

impl ExprType {
    pub fn to_string(&self) -> String {
        match self {
            ExprType::Void => "void".to_string(),
            ExprType::I32 => "i32".to_string(),
            ExprType::Bool => "bool".to_string(),
            ExprType::Str => "str".to_string(),
            ExprType::Char => "char".to_string(),
            ExprType::Struct(name) => format!("struct {}", name),
        }
    }

    pub fn from_str(s: &str) -> Option<ExprType> {
        match s {
            "void" => Some(ExprType::Void),
            "int" | "i32" => Some(ExprType::I32),
            "bool" => Some(ExprType::Bool),
            "string" => Some(ExprType::Str),
            "char" => Some(ExprType::Char),
            _ => None,
        }
    }
}
