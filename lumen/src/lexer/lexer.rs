use std::iter::Peekable;
use std::str::Chars;

use super::token_kind::TokenKind;
use super::token::Token;

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Lexer {
            chars: source.chars().peekable(),
            line: 1,
            col: 1,
        }
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.chars.next();
        if let Some(c) = c {
            if c == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        c
    }

    pub fn next_token(&mut self) -> Result<Option<Token>, String> {
        loop {
            match self.chars.peek() {
                None => return Ok(None),
                Some(c) if c.is_whitespace() => {
                    self.bump();
                }
                Some('{') => return Ok(Some(self.single_char_token(TokenKind::LBrace))),
                Some('}') => return Ok(Some(self.single_char_token(TokenKind::RBrace))),
                Some(';') => return Ok(Some(self.single_char_token(TokenKind::Semicolon))),
                Some('+') => return Ok(Some(self.single_char_token(TokenKind::Plus))),
                Some('-') => return Ok(Some(self.single_char_token(TokenKind::Minus))),
                Some('*') => return Ok(Some(self.single_char_token(TokenKind::Star))),
                Some('/') => return Ok(Some(self.single_char_token(TokenKind::Slash))),
                Some('=') => return Ok(Some(self.single_char_token(TokenKind::Equal))),
                Some('(') => return Ok(Some(self.single_char_token(TokenKind::LParen))),
                Some(')') => return Ok(Some(self.single_char_token(TokenKind::RParen))),
                Some(c) if c.is_ascii_digit() => {
                    let (line, col) = (self.line, self.col);
                    let mut text = String::new();
                    while let Some(c) = self.chars.peek() {
                        if c.is_ascii_digit() {
                            text.push(*c);
                            self.bump();
                        } else {
                            break;
                        }
                    }
                    let value: i64 = text.parse().unwrap();
                    return Ok(Some(Token { kind: TokenKind::Int(value), line, col }));
                }
                Some(c) if c.is_alphabetic() || *c == '_' => {
                    let (line, col) = (self.line, self.col);
                    let mut word = String::new();
                    while let Some(c) = self.chars.peek() {
                        if c.is_alphanumeric() || *c == '_' {
                            word.push(*c);
                            self.bump();
                        } else {
                            break;
                        }
                    }
                    let kind = match word.as_str() {
                        "return" => TokenKind::Return,
                        "var" => TokenKind::Var,
                        "val" => TokenKind::Val,
                        "if" => TokenKind::If,
                        "elif" => TokenKind::Elif,
                        "else" => TokenKind::Else,
                        _ => TokenKind::Identifier(word.clone()),
                    };
                    return Ok(Some(Token { kind, line, col }));
                }
                Some(c) => {
                    let (line, col) = (self.line, self.col);
                    let bad = *c;
                    self.bump();
                    return Err(format!("unexpected character '{bad}' (line {line}, column {col})"));
                }
            }
        }
    }

    fn single_char_token(&mut self, kind: TokenKind) -> Token {
        let (line, col) = (self.line, self.col);
        self.bump();
        Token { kind, line, col }
    }
}
