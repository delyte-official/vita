#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExprType {
    I32,
    Bool,
    Str,
    Char,
    Struct(&'static str),
}
