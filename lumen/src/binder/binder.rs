use std::collections::HashMap;
use crate::{binder::*, parser::*};
use super::{BoundedProgram, symbol::Symbol};

struct Binder {
    symbols: HashMap<String, Vec<Symbol>>,
    current_depth: usize,
    next_slot: usize,
}

impl Binder {
    fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            current_depth: 0,
            next_slot: 0,
        }
    }

    fn enter_scope(&mut self) {
        self.current_depth += 1;
    }

    fn exit_scope(&mut self) {
        for stack in self.symbols.values_mut() {
            if let Some(top) = stack.last() && top.scope_depth == self.current_depth {
                stack.pop();
            }
        }
        self.current_depth -= 1;
    }

    fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)?.last()
    }

    fn define(&mut self, name: String, ty: String, mutable: bool) -> Result<usize, String> {
        let stack = self.symbols.entry(name.clone()).or_default();
        let slot = self.next_slot;
        self.next_slot += 1;

        stack.push(Symbol {
            name,
            scope_depth: self.current_depth,
            ty,
            slot,
            mutable,
        });
        Ok(slot)
    }

    fn define_function(&mut self, name: String, index: usize) -> Result<(), String> {
        let stack = self.symbols.entry(name.clone()).or_default();
        if let Some(top) = stack.last() && top.scope_depth == self.current_depth {
            return Err(format!("function '{}' is already defined", name));
        }
        stack.push(Symbol {
            name,
            scope_depth: self.current_depth,
            ty: "i32".into(),
            slot: index,
            mutable: false,
        });
        Ok(())
    }

    fn bind_function(&mut self, func: &crate::parser::Function) -> Result<BoundedFunction, String> {
        self.next_slot = 0;
        let bounded_stmts = self.bind_block(&func.body)?;
        Ok(BoundedFunction {
            name: func.name.clone(),
            body: bounded_stmts,
            local_count: self.next_slot,
        })
    }

    fn bind_block(&mut self, stmts: &[Stmt]) -> Result<Vec<BoundedStmt>, String> {
        self.enter_scope();
        let mut bounded_stmts = vec![];
        for stmt in stmts {
            bounded_stmts.push(self.bind_stmt(stmt)?);
        }
        self.exit_scope();
        Ok(bounded_stmts)
    }

    fn bind_stmt(&mut self, stmt: &Stmt) -> Result<BoundedStmt, String> {
        match stmt {
            Stmt::Return(expr) => Ok(BoundedStmt::Return(self.bind_expr(expr)?)),
            Stmt::VarDecl(name, type_annotation, expr) => {
                let bound_expr = self.bind_expr(expr)?;
                let slot = self.define(name.clone(), "i32".into(), true)?;
                Ok(BoundedStmt::VarDecl(slot, type_annotation.clone(), bound_expr))
            }
            Stmt::ValDecl(name, type_annotation, expr) => {
                let bound_expr = self.bind_expr(expr)?;
                let slot = self.define(name.clone(), "i32".into(), false)?;
                Ok(BoundedStmt::ValDecl(slot, type_annotation.clone(), bound_expr))
            }
            Stmt::If { condition, then_branch, elif_branches, else_branch } => {
                let bound_condition = self.bind_expr(condition)?;
                let bound_then_branch = self.bind_block(then_branch)?;

                let bound_elif_branches = if let Some(elif_branches) = elif_branches {
                    let mut list = vec![];
                    for (elif_condition, elif_body) in elif_branches {
                        let bound_elif_condition = self.bind_expr(&elif_condition)?;
                        let bound_elif_body = self.bind_block(&elif_body)?;
                        list.push((bound_elif_condition, bound_elif_body));
                    }
                    Some(list)
                } else {
                    None
                };

                let bound_else_branch = match else_branch {
                    Some(stmts) => Some(self.bind_block(stmts)?),
                    None => None,
                };
                Ok(BoundedStmt::If {
                    condition: bound_condition,
                    then_branch: bound_then_branch,
                    elif_branches: bound_elif_branches,
                    else_branch: bound_else_branch,
                })
            }
            Stmt::Assign(name, expr) => {
                let bound_expr = self.bind_expr(expr)?;
                let symbol = self.lookup(name)
                    .ok_or_else(|| format!("undeclared variable '{}'", name))?;
                if !symbol.mutable {
                    return Err(format!("cannot assign to '{}': it was declared with 'val' and is not mutable", name));
                }
                Ok(BoundedStmt::Assign(symbol.slot, bound_expr))
            }
            Stmt::FuncCall { name } => {
                let symbol = self.lookup(name)
                    .ok_or_else(|| format!("undeclared function '{}'", name))?;
                Ok(BoundedStmt::FuncCall(symbol.slot))
            }
        }
    }

    fn bind_expr(&self, expr: &Expr) -> Result<BoundedExpr, String> {
        match expr {
            Expr::Literal(value) => Ok(BoundedExpr::Literal(value.clone())),
            Expr::TemplateLiteral(parts) => {
                let mut bounded_parts = vec![];
                for part in parts {
                    match part {
                        LiteralPart::Text(s) => bounded_parts.push(BoundedTemplatePart::Text(s.clone())),
                        LiteralPart::Expr(e) => bounded_parts.push(BoundedTemplatePart::Expr(Box::new(self.bind_expr(e)?))),
                    }
                }
                Ok(BoundedExpr::TemplateLiteral(bounded_parts))
            }
            Expr::Var(name) => {
                match self.lookup(name) {
                    Some(symbol) => Ok(BoundedExpr::Var(symbol.slot)),
                    None => Ok(BoundedExpr::UnresolvedName(name.clone())),
                }
            }
            Expr::FuncCall { name } => {
                let symbol = self.lookup(name)
                .ok_or_else(|| format!("undeclared function '{}'", name))?;
                Ok(BoundedExpr::FuncCall(symbol.slot))
            }
            Expr::Binary { op, left, right } => {
                let left_bound = self.bind_expr(left)?;
                let right_bound = self.bind_expr(right)?;
                Ok(BoundedExpr::Binary {
                    op: *op,
                    left: Box::new(left_bound),
                    right: Box::new(right_bound),
                })
            }
        }
    }

    fn bind_program(&mut self, program: &Program) -> Result<BoundedProgram, String> {
        for (index, func) in program.functions.iter().enumerate() {
            self.define_function(func.name.clone(), index)?;
        }

        let mut bounded_funcs: Vec<BoundedFunction> = vec![];
        for func in &program.functions {
            bounded_funcs.push(self.bind_function(func)?);
        }

        Ok(BoundedProgram {
            functions: bounded_funcs,
        })
    }
}

pub fn bind(program: Program) -> Result<BoundedProgram, String> {
    let mut binder = Binder::new();
    binder.bind_program(&program)
}
