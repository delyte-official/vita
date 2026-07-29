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

    fn parse_return_stmt(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::Return)?;
        let expr = self.parse_expr(0)?;
        self.expect(TokenKind::Semicolon)?;
        Ok(Stmt::Return(expr))
    }

    fn parse_decl_stmt(&mut self, is_mutable: bool) -> Result<Stmt, String> {
        if is_mutable {
            self.expect(TokenKind::Var)?;
        } else {
            self.expect(TokenKind::Val)?;
        }
        let name = match self.advance()? {
            Some(Token { kind: TokenKind::Identifier(name), .. }) => name,
            Some(tok) => {
                return Err(format!(
                    "expected an identifier, found {:?} (line {}, column {})",
                    tok.kind, tok.line, tok.col
                ))
            }
            None => return Err("expected an identifier, found end of file".to_string()),
        };
        self.expect(TokenKind::Equal)?;
        let expr = self.parse_expr(0)?;
        self.expect(TokenKind::Semicolon)?;
        if is_mutable {
            Ok(Stmt::VarDecl(name, expr))
        } else {
            Ok(Stmt::ValDecl(name, expr))
        }
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, String> {
        self.expect(TokenKind::LBrace)?;
        let mut stmts = vec![];
        while self.peek_kind() != Some(TokenKind::RBrace) {
            if self.current.is_none() {
                return Err("expected '}', found end of file".to_string());
            }
            stmts.push(self.parse_stmt(true)?);
        }
        self.expect(TokenKind::RBrace)?;
        Ok(stmts)
    }

    fn parse_stmt(&mut self, allow_block: bool) -> Result<Stmt, String> {
        match self.peek_kind() {
            Some(TokenKind::Return) => self.parse_return_stmt(),
            Some(TokenKind::Var) => self.parse_decl_stmt(true),
            Some(TokenKind::Val) => self.parse_decl_stmt(false),
            Some(TokenKind::If) => self.parse_if_stmt(allow_block),
            _ => Err("expected a statement".to_string()),
        }
    }

    fn parse_if_stmt(&mut self, allow_block: bool) -> Result<Stmt, String> {
        self.expect(TokenKind::If)?;
        self.expect(TokenKind::LParen)?;
        let condition = self.parse_expr(0)?;
        self.expect(TokenKind::RParen)?;
        let then_branch: Vec<Stmt>;
        if let Some(Token { kind: TokenKind::LBrace, .. }) = self.peek() {
            if !allow_block {
                return Err("blocks are not allowed here".to_string());
            }
            then_branch = self.parse_block()?;
        } else {
            then_branch = self.parse_stmt(false).map(|stmt| vec![stmt])?;
        }

        if !allow_block {
            return Ok(Stmt::If { condition, then_branch, elif_branches: None, else_branch: None })
        }

        let mut elif_branches = vec![];
        while self.peek_kind() == Some(TokenKind::Elif) {
            self.advance()?;
            self.expect(TokenKind::LParen)?;
            let elif_condition = self.parse_expr(0)?;
            self.expect(TokenKind::RParen)?;
            let elif_body: Vec<Stmt> = if let Some(Token { kind: TokenKind::LBrace, .. }) = self.peek() {
                self.parse_block()?
            } else {
                vec![self.parse_stmt(allow_block)?]
            };
            elif_branches.push((elif_condition, elif_body));
        }

        let else_branch = if self.peek_kind() == Some(TokenKind::Else) {
            self.advance()?;
            if let Some(Token { kind: TokenKind::LBrace, .. }) = self.peek() {
                Some(self.parse_block()?)
            } else {
                Some(vec![self.parse_stmt(false)?])
            }
        } else {
            None
        };

        Ok(Stmt::If { condition, then_branch, elif_branches: Some(elif_branches), else_branch })
    }

    fn parse_term(&mut self) -> Result<Expr, String> {
        match self.advance()? {
            Some(Token { kind: TokenKind::Int(n), .. }) => Ok(Expr::Int(n)),
            Some(Token { kind: TokenKind::Identifier(name), .. }) => {
                if let Some(Token { kind: TokenKind::LParen, .. }) = self.peek() {
                    self.advance()?;
                    self.expect(TokenKind::RParen)?;
                    Ok(Expr::FuncCall { name })
                } else {
                    Ok(Expr::Var(name))
                }
            }
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
                self.advance()?;
                let right = self.parse_expr(op.minimum_binding_power())?;
                left = Expr::binary(op, left, right);
        }
        Ok(left)
    }
}

pub fn parse(source: &str) -> Result<Program, String> {
    let mut parser = Parser::new(source)?;

    let mut functions = vec![];
    loop {
        let name = match parser.advance()? {
            Some(Token { kind: TokenKind::Identifier(name), .. }) => name,
            Some(tok) => {
                return Err(format!(
                    "expected a function name, found {:?} (line {}, column {})",
                    tok.kind, tok.line, tok.col
                ))
            }
            None => break,
        };
        let body = parser.parse_block()?;
        functions.push(Function { name, body });
    }

    Ok(Program { functions })
}
