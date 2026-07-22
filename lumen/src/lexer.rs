use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Main,
    Return,
    Int(i64),
    LBrace,
    RBrace,
    Semicolon,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}

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
                Some('{') => {
                    let (line, col) = (self.line, self.col);
                    self.bump();
                    return Ok(Some(Token { kind: TokenKind::LBrace, line, col }));
                }
                Some('}') => {
                    let (line, col) = (self.line, self.col);
                    self.bump();
                    return Ok(Some(Token { kind: TokenKind::RBrace, line, col }));
                }
                Some(';') => {
                    let (line, col) = (self.line, self.col);
                    self.bump();
                    return Ok(Some(Token { kind: TokenKind::Semicolon, line, col }));
                }
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
                        "main" => TokenKind::Main,
                        "return" => TokenKind::Return,
                        _ => return Err(format!("unknown word '{word}' (line {line}, column {col})")),
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
}
