use crate::{lexer::{Lexer, Token, TokenKind}, parser::*};

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

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::Return)?;
        let value = self.parse_expr(0)?;
        self.expect(TokenKind::Semicolon)?;
        Ok(Stmt::Return(value))
    }

    fn parse_term(&mut self) -> Result<Expr, String> {
        match self.advance()? {
            Some(Token { kind: TokenKind::Int(n), .. }) => Ok(Expr::Int(n)),
            Some(tok) => Err(format!(
                "expected a number, found {:?} (line {}, column {})",
                tok.kind, tok.line, tok.col
            )),
            None => Err("expected a number, found end of file".to_string()),
        }
    }

    fn parse_expr(&mut self, min_precedence: u8) -> Result<Expr, String> {
        let mut left = self.parse_term()?;
        while let Some(kind) = self.peek_kind() && let Some(op) = BinaryOp::from_token(&kind) && op.precedence() >= min_precedence {
                self.advance()?; // consume the op
                let right = self.parse_expr(op.minimum_binding_power())?;
                left = Expr::binary(op, left, right);
        }
        Ok(left)
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
        body.push(parser.parse_stmt()?);
    }

    parser.expect(TokenKind::RBrace)?;

    Ok(Program {
        main: Function {
            name: "main".to_string(),
            body,
        },
    })
}
