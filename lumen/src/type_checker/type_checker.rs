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
    let mut typed_stmts: Vec<TypedStmt> = vec![];
    for stmt in &func.body {
        match stmt {
            BoundedStmt::Return(expr) => {
                let typedexpr = check_expr(expr)?;
                if !matches!((return_type, typedexpr.ty()), (ExprType::I32, ExprType::I32)) {
                    return Err(format!("return type doesn't match function '{}' declared return type", func.name));
                }
                typed_stmts.push(TypedStmt::Return(typedexpr));
            }
            BoundedStmt::VarDecl(_, expr) | BoundedStmt::ValDecl(_, expr) => {
                let found_type = infer(expr);
                if !matches!((return_type, found_type), (ExprType::I32, ExprType::I32)) {
                    return Err(format!("variable declaration type doesn't match function '{}' declared return type", func.name));
                }
                typed_stmts.push(match stmt {
                    BoundedStmt::VarDecl(slot, _) => TypedStmt::VarDecl(*slot, check_expr(expr)?),
                    BoundedStmt::ValDecl(slot, _) => TypedStmt::ValDecl(*slot, check_expr(expr)?),
                    _ => unreachable!(),
                });
            }
        }
    }

    Ok(TypedFunction {
        name: func.name.clone(),
        body: typed_stmts,
        local_count: func.local_count,
    })
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
