use std::iter::Peekable;
use std::str::Chars;

use super::token_kind::TokenKind;
use super::token::Token;

enum LexerMode {
    ExprStart,
    AfterValue,
}

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    line: usize,
    col: usize,
    mode: LexerMode,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Lexer {
            chars: source.chars().peekable(),
            line: 1,
            col: 1,
            mode: LexerMode::ExprStart,
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
            match self.chars.peek().copied() {
                None => return Ok(None),
                Some(c) if c.is_whitespace() => {
                    self.bump();
                }
                Some('/') => {
                    let mut lookahead = self.chars.clone();
                    lookahead.next();
                    if let Some('/') = lookahead.peek() {
                        self.bump(); // consume both slashes
                        self.bump();
                        // consume rest of line
                        while let Some(c) = self.chars.peek().copied() {
                            if c == '\n' {
                                self.bump();
                                break;
                            }
                            self.bump();
                        }
                        continue;
                    }

                    // if we're expecting an expression, then its a regex
                    if matches!(self.mode, LexerMode::ExprStart) {
                        return self.lex_regex();
                    }

                    return Ok(Some(self.single_char_token(TokenKind::Slash)));
                }
                // handle literals
                Some(c) if self.is_literal_start(c) => {
                    let (line, col) = (self.line, self.col);
                    let mut text = String::new();
                    // consume first char
                    text.push(c);
                    self.bump();
                    // state
                    let mut in_tag = c.is_ascii_alphabetic() || c == '_';
                    let mut last_was_op = self.is_operator_allowed_in_literal(c);
                    while let Some(next_c) = self.chars.peek().copied() {
                        if in_tag {
                            // in trailing tag, only alpha + _
                            if next_c.is_ascii_alphabetic() || next_c == '_' {
                                text.push(next_c);
                                self.bump();
                            } else {
                                break;
                            }
                        } else {
                            // still in body
                            if next_c.is_ascii_digit() {
                                text.push(next_c);
                                self.bump();
                                last_was_op = false;
                            } else if self.is_operator_allowed_in_literal(next_c) {
                                // prevent following operators
                                if last_was_op {
                                    break;
                                }
                                let mut cloned_chars = self.chars.clone();
                                cloned_chars.next(); // skip the current peeked operator
                                let follows_operator = cloned_chars.peek();
                                let is_followed_by_valid = match follows_operator {
                                    Some(following_char) => following_char.is_ascii_digit(),
                                    None => false,
                                };

                                if !is_followed_by_valid { // not accepted
                                    break;
                                }
                                text.push(next_c);
                                self.bump();
                                last_was_op = true;
                            } else if next_c.is_ascii_alphabetic() {
                                // letter => tag
                                in_tag = true;
                                text.push(next_c);
                                self.bump();
                            } else {
                                break;
                            }
                        }
                    }
                    return self.match_keyword_or_literal(text, line, col);
                }
                Some('{') => return Ok(Some(self.single_char_token(TokenKind::LBrace))),
                Some('}') => return Ok(Some(self.single_char_token(TokenKind::RBrace))),
                Some('.') => return Ok(Some(self.single_char_token(TokenKind::Dot))),
                Some(':') => return Ok(Some(self.single_char_token(TokenKind::Colon))),
                Some(';') => return Ok(Some(self.single_char_token(TokenKind::Semicolon))),
                Some('+') => return Ok(Some(self.single_char_token(TokenKind::Plus))),
                Some('-') => return Ok(Some(self.single_char_token(TokenKind::Minus))),
                Some('*') => return Ok(Some(self.single_char_token(TokenKind::Star))),
                Some('=') => return Ok(Some(self.single_char_token(TokenKind::Equal))),
                Some('(') => return Ok(Some(self.single_char_token(TokenKind::LParen))),
                Some(')') => return Ok(Some(self.single_char_token(TokenKind::RParen))),
                Some(c) => {
                    let (line, col) = (self.line, self.col);
                    let bad = c;
                    self.bump();
                    return Err(format!("unexpected character '{bad}' (line {line}, column {col})"));
                }
            }
        }
    }

    fn single_char_token(&mut self, kind: TokenKind) -> Token {
        let (line, col) = (self.line, self.col);
        self.bump();
        self.finish(kind, line, col)
    }

    fn finish(&mut self, kind: TokenKind, line: usize, col: usize) -> Token {
        self.mode = match &kind {
            TokenKind::Literal(_) | TokenKind::RParen => LexerMode::AfterValue,
            _ => LexerMode::ExprStart,
        };
        Token { kind, line, col }
    }

    fn lex_regex(&mut self) -> Result<Option<Token>, String> {
        let (line, col) = (self.line, self.col);
        let mut raw = String::new();
        raw.push('/');
        self.bump();

        loop {
            match self.chars.peek().copied() {
                None | Some('\n') => {
                    return Err(format!("unterminated regex literal (line {line}, column {col})"));
                }
                // escaping
                Some('\\') => {
                    raw.push('\\');
                    self.bump();
                    match self.chars.peek().copied() {
                        Some(escaped) => {
                            raw.push(escaped);
                            self.bump();
                        }
                        None => {
                            return Err(format!("unterminated regex literal (line {line}, column {col})"));
                        }
                    }
                }
                // end of regex
                Some('/') => {
                    raw.push('/');
                    self.bump();
                    break;
                }
                Some(c) => {
                    raw.push(c);
                    self.bump();
                }
            }
        }

        while let Some(c) = self.chars.peek().copied() {
            if c.is_ascii_alphabetic() {
                raw.push(c);
                self.bump();
            } else {
                break;
            }
        }

        Ok(Some(self.finish(TokenKind::Literal(raw), line, col)))
    }

    fn is_literal_start(&self, c: char) -> bool {
        if c.is_ascii_alphanumeric() || c == '_' {
            return true;
        }
        if self.is_operator_allowed_in_literal(c) {
            return matches!(self.peek_second(), Some(next) if next.is_ascii_digit());
        }
        false
    }

    fn peek_second(&self) -> Option<char> {
        let mut cloned = self.chars.clone();
        cloned.next();
        cloned.peek().copied()
    }

    fn is_operator_allowed_in_literal(&self, c: char) -> bool {
        c == '.' || c == ':' // hand picked operators allowed
    }

    fn match_keyword_or_literal(
        &mut self,
        text: String,
        line: usize,
        col: usize,
    ) -> Result<Option<Token>, String> {
        let kind = match text.as_str() {
            "return" => TokenKind::Return,
            "var" => TokenKind::Var,
            "val" => TokenKind::Val,
            "if" => TokenKind::If,
            "elif" => TokenKind::Elif,
            "else" => TokenKind::Else,
            _ => TokenKind::Literal(text),
        };
        Ok(Some(self.finish(kind, line, col)))
    }

    pub fn is_identifier(&self, text: &str) -> bool {
        let mut chars = text.chars();
        match chars.next() {
            Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
            _ => return false,
        }
        chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
    }
}
