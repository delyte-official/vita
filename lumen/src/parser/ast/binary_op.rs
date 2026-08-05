use std::fmt;

use crate::lexer::TokenKind;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Plus,
    Minus,
    Star,
    Slash,
}

impl BinaryOp {
    pub const fn from_token(kind: &TokenKind) -> Option<Self> {
        match kind {
            TokenKind::Plus => Some(Self::Plus),
            TokenKind::Minus => Some(Self::Minus),
            TokenKind::Star => Some(Self::Star),
            TokenKind::Slash => Some(Self::Slash),
            _ => None,
        }
    }

    pub const fn precedence(self) -> u8 {
        match self {
            BinaryOp::Plus | BinaryOp::Minus => 1,
            BinaryOp::Star | BinaryOp::Slash => 2,
        }
    }

    pub const fn minimum_binding_power(self) -> u8 {
        match self {
            BinaryOp::Plus | BinaryOp::Minus => 2,
            BinaryOp::Star | BinaryOp::Slash => 3,
        }
    }
}

impl fmt::Debug for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op_str = match self {
            BinaryOp::Plus => "+",
            BinaryOp::Minus => "-",
            BinaryOp::Star => "*",
            BinaryOp::Slash => "/",
        };
        write!(f, "{}", op_str)
    }
}