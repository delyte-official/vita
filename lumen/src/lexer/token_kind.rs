#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Return,
    Int(i64),
    LBrace,
    RBrace,
    Semicolon,
    Plus,
    Minus,
    Star,
    Slash,
    Var,
    Val,
    Identifier(String),
    Equal,
}