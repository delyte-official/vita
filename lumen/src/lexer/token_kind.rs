#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Main,
    Return,
    Int(i64),
    LBrace,
    RBrace,
    Semicolon,
    Plus,
    Minus,
    Star,
    Slash,
}