use std::iter::Peekable;
use std::str::Chars;

use super::token_kind::TokenKind;
use super::token::Token;

enum LexerMode {
    ExprStart,
    AfterValue,
}

enum TokenMode {
    Normal,
    InlineString,
    MultilineString,
}

enum BraceKind {
    Block,
    LiteralHole(TokenMode),
}

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    line: usize,
    col: usize,
    state_stack: Vec<LexerMode>,
    brace_kinds: Vec<BraceKind>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Lexer {
            chars: source.chars().peekable(),
            line: 1,
            col: 1,
            state_stack: Vec::new(),
            brace_kinds: Vec::new(),
        }
    }

    fn mode(&self) -> &LexerMode {
        self.state_stack.last().unwrap_or(&LexerMode::ExprStart)
    }

    fn set_mode(&mut self, mode: LexerMode) {
        self.state_stack.push(mode);
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
                    if matches!(self.mode(), LexerMode::ExprStart) {
                        return self.lex_regex();
                    }

                    return Ok(Some(self.single_char_token(TokenKind::Slash)));
                }
                // handle literals
                Some(c) if c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '`' || c == '\'' || c == '"' => {
                    let (line, col) = (self.line, self.col);
                    let text = String::new();
                    return self.continue_literal_scan(text, line, col, false, TokenMode::Normal);
                }
                Some('{') => {
                    self.brace_kinds.push(BraceKind::Block);
                    return Ok(Some(self.single_char_token(TokenKind::LBrace)));
                }
                Some('}') => {
                    let (line, col) = (self.line, self.col);
                    self.bump();
                    match self.brace_kinds.pop() {
                        Some(BraceKind::LiteralHole(token_mode)) => return self.continue_literal_scan(String::new(), line, col, true, token_mode),
                        _ => return Ok(Some(self.finish(TokenKind::RBrace, line, col))),
                    }
                }
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

    fn continue_literal_scan(&mut self, mut text: String, line: usize, col: usize, is_resume: bool, token_mode: TokenMode) -> Result<Option<Token>, String> {
        match token_mode {
            TokenMode::InlineString => {
                let interrupted = self.lex_inline_string_body(&mut text, line, col, is_resume)?;
                if let Some(token) = interrupted {
                    return Ok(Some(token));
                }
            }
            TokenMode::MultilineString => {
                let interrupted = self.lex_multiline_string_body(&mut text, line, col, is_resume)?;
                if let Some(token) = interrupted {
                    return Ok(Some(token));
                }
            }
            TokenMode::Normal => {}
        }

        while let Some(next_c) = self.chars.peek().copied() {
            if next_c.is_ascii_alphanumeric() || next_c == '_' {
                text.push(next_c);
                self.bump();
            } // handling the colon rule: must not be the end of the literal
            else if next_c == ':' {
                let mut cloned_chars = self.chars.clone();
                cloned_chars.next(); // skip the current peeked operator
                let follows_colon = cloned_chars.peek();
                let is_followed_by_valid = match follows_colon {
                    Some(following_char) => following_char.is_ascii_alphanumeric() || *following_char == '_' || *following_char == ':' || *following_char == '.' || *following_char == '`' || *following_char == '\'' || *following_char == '"' || *following_char == '{',
                    None => false,
                };

                if !is_followed_by_valid { // not accepted
                    break;
                }
                text.push(next_c);
                self.bump();
            } // handling the dot rule: must be followed by a digit
            else if next_c == '.' {
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
            } // handling the <char> atom, of the form 'a' or '\n'
            else if next_c == '\'' {
                let char_atom = self.lex_char_atom()?;
                text.push_str(&char_atom);
            } // handling backticks (either single backticks => raw string, or triple backticks => multi-line raw string)
            else if next_c == '`' {
                let mut cloned_chars = self.chars.clone();
                cloned_chars.next();
                let second_backtick = cloned_chars.next();
                let third_backtick = cloned_chars.clone().next(); // peek another step ahead
                if second_backtick == Some('`') && third_backtick == Some('`') { // multi-line raw string
                    let multiline_raw_string_atom = self.lex_multiline_raw_string()?;
                    text.push_str(&multiline_raw_string_atom);
                } else { // inline raw string
                    let inline_raw_string_atom = self.lex_inline_raw_string()?;
                    text.push_str(&inline_raw_string_atom);
                }
            } // normal strings
            else if next_c == '"' {
                let mut cloned_chars = self.chars.clone();
                cloned_chars.next();
                let second_quote = cloned_chars.next();
                let third_quote = cloned_chars.clone().next(); // peek another step ahead
                let interrupted = if second_quote == Some('"') && third_quote == Some('"') { // multi-line string
                    self.lex_multiline_string(&mut text, line, col, is_resume)?
                } else { // inline string
                    self.lex_inline_string(&mut text, line, col, is_resume)?
                };
                if let Some(token) = interrupted {
                    return Ok(Some(token));
                }
            }
            // handling braces
            else if next_c == '{' {
                self.bump(); // consume the opening brace
                self.brace_kinds.push(BraceKind::LiteralHole(token_mode));
                //give back control to parser
                let kind = if is_resume {
                    TokenKind::LiteralMiddleTemplate(text)
                } else {
                    TokenKind::LiteralStartTemplate(text)
                };
                return Ok(Some(self.finish(kind, line, col)));
            }
            else {
                break;
            }
        }

        if is_resume {
            Ok(Some(self.finish(TokenKind::LiteralEndTemplate(text), line, col)))
        } else {
            self.match_keyword_or_literal(text, line, col)
        }
    }

    fn single_char_token(&mut self, kind: TokenKind) -> Token {
        let (line, col) = (self.line, self.col);
        self.bump();
        self.finish(kind, line, col)
    }

    fn finish(&mut self, kind: TokenKind, line: usize, col: usize) -> Token {
        self.set_mode(match &kind {
            TokenKind::Literal(_) | TokenKind::LiteralEndTemplate(_) | TokenKind::RParen => LexerMode::AfterValue,
            _ => LexerMode::ExprStart,
        });
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

        // match regex flags
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
            "func" => TokenKind::Func,
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

    fn lex_char_atom(&mut self) -> Result<String, String> {
        let (line, col) = (self.line, self.col);
        let mut raw = String::new();
        raw.push('\'');
        self.bump(); // consume opening quote

        match self.chars.peek().copied() {
            Some('\\') => {
                raw.push('\\');
                self.bump();
                match self.chars.peek().copied() {
                    Some(escaped) => {
                        raw.push(escaped);
                        self.bump();
                    }
                    None => {
                        return Err(format!("unterminated char atom (line {line}, column {col})"));
                    }
                }
            }
            Some(c) => {
                raw.push(c);
                self.bump();
            }
            None => {
                return Err(format!("unterminated char atom (line {line}, column {col})"));
            }
        }

        // closing quote
        match self.chars.peek().copied() {
            Some('\'') => {
                raw.push('\'');
                self.bump(); // consume the closing quote
            }
            _ => {
                return Err(format!("unterminated char atom (line {line}, column {col})"));
            }
        }

        Ok(raw)
    }

    fn lex_multiline_raw_string(&mut self) -> Result<String, String> {
        let (line, col) = (self.line, self.col);
        let mut raw = String::new();
        raw.push_str("```");
        self.bump();
        self.bump();
        self.bump();
        loop {
            match self.chars.peek().copied() {
                None => {
                    return Err(format!("unterminated multiline raw string (line {line}, column {col})"));
                }
                Some('`') => {
                    let mut cloned_chars = self.chars.clone();
                    cloned_chars.next();
                    let second_backtick = cloned_chars.next();
                    let third_backtick = cloned_chars.clone().next(); // peek another step ahead
                    if second_backtick == Some('`') && third_backtick == Some('`') {
                        raw.push_str("```");
                        self.bump();
                        self.bump();
                        self.bump();
                        break;
                    } else {
                        raw.push('`');
                    }
                }
                Some(c) => {
                    raw.push(c);
                }
            }
            self.bump();
        }

        Ok(raw)
    }

    fn lex_inline_raw_string(&mut self) -> Result<String, String> {
        let (line, col) = (self.line, self.col);
        let mut raw = String::new();
        raw.push('`');
        self.bump(); // consume opening backtick

        loop {
            match self.chars.peek().copied() {
                None | Some('\n') => {
                    return Err(format!("unterminated inline raw string (line {line}, column {col})"));
                }
                Some('`') => {
                    raw.push('`');
                    self.bump(); // consume closing backtick
                    break;
                }
                Some(c) => {
                    raw.push(c);
                    self.bump();
                }
            }
        }

        Ok(raw)
    }

    fn lex_multiline_string(&mut self, text: &mut String, line: usize, col: usize, is_resume: bool) -> Result<Option<Token>, String> {
        text.push_str("\"\"\"");
        self.bump();
        self.bump();
        self.bump();
        self.lex_multiline_string_body(text, line, col, is_resume)
    }

    fn lex_multiline_string_body(&mut self, text: &mut String, line: usize, col: usize, is_resume: bool) -> Result<Option<Token>, String> {
        loop {
            match self.chars.peek().copied() {
                None => {
                    return Err(format!("unterminated multiline string (line {line}, column {col})"));
                }
                // escaping
                Some('\\') => {
                    text.push('\\');
                    self.bump();
                    match self.chars.peek().copied() {
                        Some(escaped) => {
                            text.push(escaped);
                            self.bump();
                        }
                        None => {
                            return Err(format!("unterminated char atom (line {line}, column {col})"));
                        }
                    }
                }
                Some('{') => {
                    self.bump(); // consume the opening brace
                    self.brace_kinds.push(BraceKind::LiteralHole(TokenMode::MultilineString));
                    let kind = if is_resume {
                        TokenKind::LiteralMiddleTemplate(std::mem::take(text))
                    } else {
                        TokenKind::LiteralStartTemplate(std::mem::take(text))
                    };
                    return Ok(Some(self.finish(kind, line, col)));
                }
                Some('"') => {
                    let mut cloned_chars = self.chars.clone();
                    cloned_chars.next();
                    let second_quote = cloned_chars.next();
                    let third_quote = cloned_chars.clone().next(); // peek another step ahead
                    if second_quote == Some('"') && third_quote == Some('"') {
                        text.push_str("\"\"\"");
                        self.bump();
                        self.bump();
                        self.bump();
                        return Ok(None);
                    } else {
                        text.push('"');
                        self.bump();
                    }
                }
                Some(c) => {
                    text.push(c);
                    self.bump();
                }
            }
        }
    }

    fn lex_inline_string(&mut self, text: &mut String, line: usize, col: usize, is_resume: bool) -> Result<Option<Token>, String> {
        text.push('"');
        self.bump(); // consume opening quote
        self.lex_inline_string_body(text, line, col, is_resume)
    }

    fn lex_inline_string_body(&mut self, text: &mut String, line: usize, col: usize, is_resume: bool) -> Result<Option<Token>, String> {
        loop {
            match self.chars.peek().copied() {
                None | Some('\n') => {
                    return Err(format!("unterminated inline string (line {line}, column {col})"));
                }
                // escaping
                Some('\\') => {
                    text.push('\\');
                    self.bump();
                    match self.chars.peek().copied() {
                        Some(escaped) => {
                            text.push(escaped);
                            self.bump();
                        }
                        None => {
                            return Err(format!("unterminated char atom (line {line}, column {col})"));
                        }
                    }
                }
                Some('{') => {
                    self.bump(); // consume the opening brace
                    self.brace_kinds.push(BraceKind::LiteralHole(TokenMode::InlineString));
                    let kind = if is_resume {
                        TokenKind::LiteralMiddleTemplate(std::mem::take(text))
                    } else {
                        TokenKind::LiteralStartTemplate(std::mem::take(text))
                    };
                    return Ok(Some(self.finish(kind, line, col)));
                }
                Some('"') => {
                    text.push('"');
                    self.bump(); // consume closing quote
                    return Ok(None);
                }
                Some(c) => {
                    text.push(c);
                    self.bump();
                }
            }
        }
    }
}
