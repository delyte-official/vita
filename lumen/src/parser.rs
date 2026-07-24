use crate::lexer::{Lexer, Token, TokenKind};

#[derive(Debug)]
pub struct Program {
    pub main: Function,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub body: Vec<Stmt>,
}

#[derive(Debug)]
pub enum Stmt {
    Return(Expr),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug)]
pub enum Expr {
    Int(i64),
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

impl Expr {
    pub fn binary(op: BinaryOp, left: Expr, right: Expr) -> Expr {
        Expr::Binary { op, left: Box::new(left), right: Box::new(right) }
    }
}

struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Option<Token>,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str) -> Result<Self, String> {
        let mut lexer = Lexer::new(source);
        let current = lexer.next_token()?;
        Ok(Parser { lexer, current })
    }

    fn advance(&mut self) -> Result<Option<Token>, String> {
        let tok = self.current.take();
        self.current = self.lexer.next_token()?;
        Ok(tok)
    }

    fn peek(&self) -> Option<&Token> {
        self.current.as_ref()
    }

    fn peek_kind(&self) -> Option<TokenKind> {
        self.peek().map(|t| t.kind.clone())
    }

    fn expect(&mut self, expected: TokenKind) -> Result<(), String> {
        match self.advance()? {
            Some(tok) if tok.kind == expected => Ok(()),
            Some(tok) => Err(format!(
                "expected {expected:?}, found {:?} (line {}, column {})",
                tok.kind, tok.line, tok.col
            )),
            None => Err(format!("expected {expected:?}, found end of file")),
        }
    }
}

pub fn parse(source: &str) -> Result<Program, String> {
    let mut parser = Parser::new(source)?;

    parser.expect(TokenKind::Main)?;
    parser.expect(TokenKind::LBrace)?;

    let mut body = vec![];
    while parser.peek_kind() != Some(TokenKind::RBrace) {
        if parser.current.is_none() {
            return Err("expected '}', found end of file".to_string());
        }
        body.push(parse_stmt(&mut parser)?);
    }

    parser.expect(TokenKind::RBrace)?;

    Ok(Program {
        main: Function {
            name: "main".to_string(),
            body,
        },
    })
}

fn parse_stmt(parser: &mut Parser) -> Result<Stmt, String> {
    parser.expect(TokenKind::Return)?;
    let value = parse_expr(parser, 0)?;
    parser.expect(TokenKind::Semicolon)?;
    Ok(Stmt::Return(value))
}

fn parse_term(parser: &mut Parser) -> Result<Expr, String> {
    match parser.advance()? {
        Some(Token { kind: TokenKind::Int(n), .. }) => Ok(Expr::Int(n)),
        Some(tok) => Err(format!(
            "expected a number, found {:?} (line {}, column {})",
            tok.kind, tok.line, tok.col
        )),
        None => Err("expected a number, found end of file".to_string()),
    }
}

fn parse_expr(parser: &mut Parser, min_precedence: u8) -> Result<Expr, String> {
    let mut left = parse_term(parser)?;
    while let Some(kind) = parser.peek_kind() && let Some(op) = BinaryOp::from_token(&kind) && op.precedence() >= min_precedence {
            parser.advance()?; // consume the op
            let right = parse_expr(parser, op.minimum_binding_power())?;
            left = Expr::binary(op, left, right);
    }
    Ok(left)
}
