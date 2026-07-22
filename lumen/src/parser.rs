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

#[derive(Debug)]
pub enum Expr {
    Int(i64),
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
    while parser.current.as_ref().map(|t| &t.kind) != Some(&TokenKind::RBrace) {
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
    let value = parse_expr(parser)?;
    parser.expect(TokenKind::Semicolon)?;
    Ok(Stmt::Return(value))
}

fn parse_expr(parser: &mut Parser) -> Result<Expr, String> {
    match parser.advance()? {
        Some(Token { kind: TokenKind::Int(n), .. }) => Ok(Expr::Int(n)),
        Some(tok) => Err(format!(
            "expected a number, found {:?} (line {}, column {})",
            tok.kind, tok.line, tok.col
        )),
        None => Err("expected a number, found end of file".to_string()),
    }
}
