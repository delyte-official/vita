use crate::{lexer::{Lexer, Token, TokenKind}, parser::{PrimitiveClassItem, program::Definition, *}};

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

    fn peek_line_col(&self) -> (usize, usize) {
        self.peek().map_or((0, 0), |t| (t.line, t.col))
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

    fn parse_identifier(&mut self) -> Result<String, String> {
        match self.advance()? {
            Some(Token { kind: TokenKind::Literal(value), .. }) if self.lexer.is_identifier(&value) => Ok(value),
            Some(Token { kind: TokenKind::Literal(value), line, col }) => Err(format!(
                "expected an identifier, found literal '{value}' (line {line}, column {col})"
            )),
            Some(tok) => Err(format!(
                "expected an identifier, found {:?} (line {}, column {})",
                tok.kind, tok.line, tok.col
            )),
            None => Err("expected an identifier, found end of file".to_string()),
        }
    }

    fn parse_decl_stmt(&mut self, is_mutable: bool) -> Result<Stmt, String> {
        if is_mutable {
            self.expect(TokenKind::Var)?;
        } else {
            self.expect(TokenKind::Val)?;
        }
        let name = self.parse_identifier()?;

        let type_annotation = if self.peek_kind() == Some(TokenKind::Colon) {
            self.advance()?;
            Some(self.parse_identifier()?)
        } else {
            None
        };

        self.expect(TokenKind::Equal)?;
        let expr = self.parse_expr(0)?;
        self.expect(TokenKind::Semicolon)?;
        if is_mutable {
            Ok(Stmt::VarDecl(name, type_annotation, expr))
        } else {
            Ok(Stmt::ValDecl(name, type_annotation, expr))
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
            Some(TokenKind::Literal(value)) => {
                let (lit_line, lit_col) = self.peek_line_col();
                if !self.lexer.is_identifier(&value) {
                    return Err(format!(
                        "expected a statement, found literal '{value}' (line {lit_line}, column {lit_col})"
                    ));
                }
                self.advance()?;
                match self.peek_kind() {
                    Some(TokenKind::Equal) => {
                        self.advance()?;
                        let expr = self.parse_expr(0)?;
                        self.expect(TokenKind::Semicolon)?;
                        Ok(Stmt::Assign(value, expr))
                    }
                    Some(TokenKind::LParen) => {
                        self.advance()?;
                        self.expect(TokenKind::RParen)?;
                        self.expect(TokenKind::Semicolon)?;
                        Ok(Stmt::FuncCall { name: value })
                    }
                    _ => Err(format!(
                        "expected '=' or '(', found {:?} (line {}, column {})",
                        self.peek_kind(),
                        self.peek().map_or(0, |t| t.line),
                        self.peek().map_or(0, |t| t.col)
                    )),
                }
            }
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
                vec![self.parse_stmt(false)?]
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
            Some(Token { kind: TokenKind::Literal(text), .. }) => {
                if self.lexer.is_identifier(&text) {
                    if let Some(Token { kind: TokenKind::LParen, .. }) = self.peek() {
                        self.advance()?;
                        self.expect(TokenKind::RParen)?;
                        Ok(Expr::FuncCall { name: text })
                    } else {
                        Ok(Expr::Var(text))
                    }
                } else {
                    Ok(Expr::Literal(text))
                }
            }
            Some(Token { kind: TokenKind::LiteralStartTemplate(text), .. }) => self.parse_format_literal(text),
            Some(Token { kind: TokenKind::LParen, .. }) => {
                let expr = self.parse_expr(0)?;
                self.expect(TokenKind::RParen)?;
                Ok(Expr::Parenthesized(Box::new(expr)))
            }
            Some(tok) => Err(format!(
                "expected an expression, found {:?} (line {}, column {})",
                tok.kind, tok.line, tok.col
            )),
            None => Err("expected an expression, found end of file".to_string()),
        }
    }

    fn parse_format_literal(&mut self, first: String) -> Result<Expr, String> {
        let mut parts = vec![LiteralPart::Text(first)];
        loop {
            let hole = self.parse_expr(0)?;
            parts.push(LiteralPart::Expr(hole));
            match self.advance()? {
                Some(Token { kind: TokenKind::LiteralMiddleTemplate(text), .. }) => {
                    parts.push(LiteralPart::Text(text));
                }
                Some(Token { kind: TokenKind::LiteralEndTemplate(text), .. }) => {
                    parts.push(LiteralPart::Text(text));
                    return Ok(Expr::TemplateLiteral(parts));
                }
                Some(tok) => {
                    return Err(format!(
                        "expected the rest of the formatted literal, found {:?} (line {}, column {})",
                        tok.kind, tok.line, tok.col
                    ))
                }
                None => {
                    return Err("expected the rest of the formatted literal, found end of file".to_string())
                }
            }
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

    fn parse_function(&mut self) -> Result<Function, String> {
        self.expect(TokenKind::Func)?;
        let name = self.parse_identifier()?;
        self.expect(TokenKind::LParen)?;
        self.expect(TokenKind::RParen)?;
        let return_type = if let Some(Token { kind: TokenKind::Colon, .. }) = self.peek() {
            self.advance()?;
            Some(self.parse_identifier()?)
        } else {
            None
        };
        let body = self.parse_block()?;
        Ok(Function { name, body, return_type })
    }

    pub fn parse_field(&mut self) -> Result<Field, String> {
        let mutable = match self.advance()? {
            Some(Token{ kind: TokenKind::Var, .. }) => true,
            Some(Token{ kind: TokenKind::Val, .. }) => false,
            _ => return Err(format!(
                "expected 'var' or 'val', found {:?} (line {}, column {})",
                self.peek_kind(),
                self.peek().map_or(0, |t| t.line),
                self.peek().map_or(0, |t| t.col)
            )),
        };
        let name = self.parse_identifier()?;
        self.expect(TokenKind::Colon)?;
        let type_annotation = self.parse_identifier()?;
        self.expect(TokenKind::Semicolon)?;
        Ok(Field { name, ty: type_annotation, mutable })
    }

    // already consumed '<'
    pub fn parse_type_atom(&mut self) -> Result<LiteralAtom, String> {
        let name = self.parse_identifier()?;
        self.expect(TokenKind::Colon)?;
        let ty = self.parse_identifier()?;
        self.expect(TokenKind::RightAngleBracket)?;
        Ok(LiteralAtom::TypeAtom { name, ty })
    }

    // stops at the first parenthesis, bc its supposed to be only in a primitive constructor
    pub fn parse_literal_representation(&mut self) -> Result<LiteralRepresentation, String> {
        let mut literal_repr = Vec::new();
        while self.peek_kind() != Some(TokenKind::RParen) {
            let literal_atom = match self.advance()? {
                Some(Token { kind: TokenKind::LeftAngleBracket, .. }) => self.parse_type_atom()?,
                Some(Token { kind: TokenKind::Literal(value), .. }) => LiteralAtom::Text(value.clone()),
                Some(tok) => return Err(format!(
                    "expected a literal atom, found {:?} (line {}, column {})",
                    tok.kind,
                    tok.line,
                    tok.col
                )),
                None => return Err("expected a literal atom, found end of file".to_string()),
            };
            literal_repr.push(literal_atom);
        }
        Ok(LiteralRepresentation { parts: literal_repr })
    }

    pub fn parse_primitive_constructor(&mut self) -> Result<PrimitiveConstructor, String> {
        self.expect(TokenKind::Constructor)?;
        self.expect(TokenKind::LParen)?;
        let literal_repr = self.parse_literal_representation()?;
        self.expect(TokenKind::RParen)?;
        let body = self.parse_block()?;
        Ok(PrimitiveConstructor { literal_representation: literal_repr, body })
    }

    pub fn parse_primitive_class(&mut self) -> Result<PrimitiveClass, String> {
        self.expect(TokenKind::Primitive)?;
        self.expect(TokenKind::Class)?;
        let name = self.parse_identifier()?;
        self.expect(TokenKind::LBrace)?;
        let mut class_items = vec![];
        while self.peek_kind() != Some(TokenKind::RBrace) {
            let item = match self.peek_kind() {
                Some(TokenKind::Func) => PrimitiveClassItem::Method(self.parse_function()?),
                Some(TokenKind::Val) => PrimitiveClassItem::Field(self.parse_field()?),
                Some(TokenKind::Var) => return Err("fields in primitive classes cannot be mutable".to_string()),
                Some(TokenKind::Constructor) => PrimitiveClassItem::Constructor(self.parse_primitive_constructor()?),
                _ => return Err(format!(
                    "expected a method, field, or constructor, found {:?} (line {}, column {})",
                    self.peek_kind(),
                    self.peek().map_or(0, |t| t.line),
                    self.peek().map_or(0, |t| t.col)
                ))
            };
            class_items.push(item);
        }
        self.expect(TokenKind::RBrace)?;
        Ok(PrimitiveClass { name, items: class_items })
    }
}

pub fn parse(source: &str) -> Result<Program, String> {
    let mut parser = Parser::new(source)?;

    let mut definitions = vec![];
    while let Some(_) = parser.peek() {
        let definition = match parser.peek_kind() {
            Some(TokenKind::Func) => Definition::Function(parser.parse_function()?),
            Some(TokenKind::Primitive) => Definition::PrimitiveClass(parser.parse_primitive_class()?),
            _ => return Err(format!(
                "expected a definition, found {:?} (line {}, column {})",
                parser.peek_kind(),
                parser.peek().map_or(0, |t| t.line),
                parser.peek().map_or(0, |t| t.col)
            )),
        };
        definitions.push(definition);
    }

    Ok(Program { definitions })
}
