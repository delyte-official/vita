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

    fn define(&mut self, name: String, ty: String) -> Result<usize, String> {
        let stack = self.symbols.entry(name.clone()).or_default();        
        if let Some(top) = stack.last() && top.scope_depth == self.current_depth {
            return Err(format!("variable '{}' is already defined in the current scope", name));
        }
        let slot = self.next_slot;
        self.next_slot += 1;

        stack.push(Symbol {
            name,
            scope_depth: self.current_depth,
            ty,
            slot,
        });
        Ok(slot)
    }

    fn bind_function(&mut self, func: &crate::parser::Function) -> Result<BoundedFunction, String> {
        self.define(func.name.clone(), "i32".into())?;
        self.next_slot = 0;
        self.enter_scope();
        let mut bounded_stmts = vec![];

        for stmt in &func.body {
            bounded_stmts.push(self.bind_stmt(stmt)?);
        }

        self.exit_scope();
        Ok(BoundedFunction {
            name: func.name.clone(),
            body: bounded_stmts,
            local_count: self.next_slot,
        })
    }

    fn bind_stmt(&mut self, stmt: &Stmt) -> Result<BoundedStmt, String> {
        match stmt {
            Stmt::Return(expr) => Ok(BoundedStmt::Return(self.bind_expr(expr)?)),
            Stmt::VarDecl(name, expr) => {
                let bound_expr = self.bind_expr(expr)?;
                let slot = self.define(name.clone(), "i32".into())?;
                Ok(BoundedStmt::VarDecl(slot, bound_expr))
            }
            Stmt::ValDecl(name, expr) => {
                let bound_expr = self.bind_expr(expr)?;
                let slot = self.define(name.clone(), "i32".into())?;
                Ok(BoundedStmt::ValDecl(slot, bound_expr))
            }
        }
    }

    fn bind_expr(&self, expr: &Expr) -> Result<BoundedExpr, String> {
        match expr {
            Expr::Int(n) => Ok(BoundedExpr::Int(*n)),
            Expr::Var(name) => {
                let symbol = self.lookup(name)
                .ok_or_else(|| format!("undeclared variable '{}'", name))?;
                Ok(BoundedExpr::Var(symbol.slot))
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