use crate::binder::*;
use crate::type_checker::*;

pub fn check(program: BoundedProgram) -> Result<TypedProgram, String> {
    let mut typed_functions: Vec<TypedFunction> = vec![];
    for func in &program.functions {
        typed_functions.push(check_function(func)?);
    }

    Ok(TypedProgram {
        functions: typed_functions,
    })
}

fn check_function(func: &BoundedFunction) -> Result<TypedFunction, String> {
    let return_type = ExprType::I32;
    let typed_stmts = check_stmts(&func.body, return_type, &func.name)?;

    if !stmts_always_return(&func.body) {
        return Err(format!(
            "function '{}' does not return a value on all code paths",
            func.name
        ));
    }

    Ok(TypedFunction {
        name: func.name.clone(),
        body: typed_stmts,
        local_count: func.local_count,
    })
}

fn stmts_always_return(stmts: &[BoundedStmt]) -> bool {
    stmts.iter().any(stmt_always_returns)
}

fn stmt_always_returns(stmt: &BoundedStmt) -> bool {
    match stmt {
        BoundedStmt::Return(_) => true,
        BoundedStmt::VarDecl(_, _) | BoundedStmt::ValDecl(_, _) => false,
        BoundedStmt::If { then_branch, elif_branches, else_branch, .. } => {
            let Some(else_branch) = else_branch else {
                return false;
            };

            let then_ok = stmts_always_return(then_branch);
            let elifs_ok = elif_branches
                .as_ref()
                .map(|branches| branches.iter().all(|(_, body)| stmts_always_return(body)))
                .unwrap_or(true);
            let else_ok = stmts_always_return(else_branch);

            then_ok && elifs_ok && else_ok
        }
    }
}

fn check_stmts(stmts: &[BoundedStmt], return_type: ExprType, func_name: &str) -> Result<Vec<TypedStmt>, String> {
    stmts.iter().map(|stmt| check_stmt(stmt, return_type, func_name)).collect()
}

fn check_stmt(stmt: &BoundedStmt, return_type: ExprType, func_name: &str) -> Result<TypedStmt, String> {
    match stmt {
        BoundedStmt::Return(expr) => {
            let typedexpr = check_expr(expr)?;
            if !matches!((return_type, typedexpr.ty()), (ExprType::I32, ExprType::I32)) {
                return Err(format!("return type doesn't match function '{}' declared return type", func_name));
            }
            Ok(TypedStmt::Return(typedexpr))
        }
        BoundedStmt::VarDecl(slot, expr) => {
            let found_type = infer(expr);
            if !matches!((return_type, found_type), (ExprType::I32, ExprType::I32)) {
                return Err(format!("variable declaration type doesn't match function '{}' declared return type", func_name));
            }
            Ok(TypedStmt::VarDecl(*slot, check_expr(expr)?))
        }
        BoundedStmt::ValDecl(slot, expr) => {
            let found_type = infer(expr);
            if !matches!((return_type, found_type), (ExprType::I32, ExprType::I32)) {
                return Err(format!("variable declaration type doesn't match function '{}' declared return type", func_name));
            }
            Ok(TypedStmt::ValDecl(*slot, check_expr(expr)?))
        }
        BoundedStmt::If { condition, then_branch, elif_branches, else_branch } => {
            let condition_type = infer(condition);
            if !matches!(condition_type, ExprType::I32) {
                return Err(format!("if condition in function '{}' must be i32", func_name));
            }
            let typed_condition = check_expr(condition)?;
            let typed_then = check_stmts(then_branch, return_type, func_name)?;

            let typed_elif_branches = if let Some(elif_branches) = elif_branches {
                let mut typed_elif_branches = Vec::new();
                for (elif_condition, elif_body) in elif_branches {
                    let elif_condition_type = infer(elif_condition);
                    if !matches!(elif_condition_type, ExprType::I32) {
                        return Err(format!("elif condition in function '{}' must be i32", func_name));
                    }
                    let typed_elif_condition = check_expr(elif_condition)?;
                    let typed_elif_body = check_stmts(elif_body, return_type, func_name)?;
                    typed_elif_branches.push((typed_elif_condition, typed_elif_body));
                }
                Some(typed_elif_branches)
            } else {
                None
            };

            let typed_else = match else_branch {
                Some(stmts) => Some(check_stmts(stmts, return_type, func_name)?),
                None => None,
            };
            Ok(TypedStmt::If {
                condition: typed_condition,
                then_branch: typed_then,
                elif_branches: typed_elif_branches,
                else_branch: typed_else,
            })
        }
    }
}

fn check_expr(expr: &BoundedExpr) -> Result<TypedExpr, String> {
    match expr {
        BoundedExpr::Int(n) => Ok(TypedExpr::Int(*n)),
        BoundedExpr::Binary { op, left, right } => {
            let left_type = infer(left);
            let right_type = infer(right);
            if !matches!((left_type, right_type), (ExprType::I32, ExprType::I32)) {
                return Err("binary operation operands must be of type i32".to_string());
            }
            Ok(TypedExpr::Binary {
                op: *op,
                left: Box::new(check_expr(left)?),
                right: Box::new(check_expr(right)?),
                ty: ExprType::I32,
            })
        }
        BoundedExpr::Var(slot) => Ok(TypedExpr::Var(*slot)),
        BoundedExpr::FuncCall(slot) => Ok(TypedExpr::FuncCall(*slot)),
    }
}

fn infer(expr: &BoundedExpr) -> ExprType {
    match expr {
        BoundedExpr::Int(_) => ExprType::I32,
        BoundedExpr::Binary { left, right, .. } => {
            let left_type = infer(left);
            let right_type = infer(right);
            if !matches!((left_type, right_type), (ExprType::I32, ExprType::I32)) {
                panic!("binary operation operands must be of type i32");
            }
            ExprType::I32
        }
        BoundedExpr::Var(_) => ExprType::I32,
        BoundedExpr::FuncCall(_) => ExprType::I32,
    }
}
