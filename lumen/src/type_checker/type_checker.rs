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
        BoundedStmt::VarDecl(_, _, _) | BoundedStmt::ValDecl(_, _, _) => false,
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
        BoundedStmt::Assign(_, _) => false,
        BoundedStmt::FuncCall(_) => false,
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
        BoundedStmt::VarDecl(slot, type_annotation, expr) => {
            let typed_expr = check_expr(expr)?;
            if let Some(type_name) = type_annotation {
                let declared_type = resolve_type_name(type_name)?;
                if declared_type != typed_expr.ty() {
                    return Err(format!("variable declared as '{type_name}' but initializer has a different type"));
                }
            }
            Ok(TypedStmt::VarDecl(*slot, typed_expr))
        }
        BoundedStmt::ValDecl(slot, type_annotation, expr) => {
            let typed_expr = check_expr(expr)?;
            if let Some(type_name) = type_annotation {
                let declared_type = resolve_type_name(type_name)?;
                if declared_type != typed_expr.ty() {
                    return Err(format!("variable declared as '{type_name}' but initializer has a different type"));
                }
            }
            Ok(TypedStmt::ValDecl(*slot, typed_expr))
        }
        BoundedStmt::If { condition, then_branch, elif_branches, else_branch } => {
            let condition_type = infer(condition)?;
            if !matches!(condition_type, ExprType::I32) {
                return Err(format!("if condition in function '{}' must be i32", func_name));
            }
            let typed_condition = check_expr(condition)?;
            let typed_then = check_stmts(then_branch, return_type, func_name)?;

            let typed_elif_branches = if let Some(elif_branches) = elif_branches {
                let mut typed_elif_branches = Vec::new();
                for (elif_condition, elif_body) in elif_branches {
                    let elif_condition_type = infer(elif_condition)?;
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
        BoundedStmt::Assign(slot, expr) => Ok(TypedStmt::Assign(*slot, check_expr(expr)?)),
        BoundedStmt::FuncCall(slot) => Ok(TypedStmt::FuncCall(*slot)),
    }
}

fn resolve_type_name(name: &str) -> Result<ExprType, String> {
    match name {
        "int" | "i32" => Ok(ExprType::I32),
        "bool" => Ok(ExprType::Bool),
        "string" => Ok(ExprType::Str),
        "char" => Ok(ExprType::Char),
        _ => Err(format!("unknown type '{name}'")),
    }
}

fn resolve_zero_arity_tag(name: &str) -> Result<TypedExpr, String> {
    match name {
        "true" | "false" => Ok(TypedExpr::Literal { value: name.to_string(), ty: ExprType::Bool }),
        _ => Err(format!("undeclared variable '{}'", name)),
    }
}

fn infer_zero_arity_tag(name: &str) -> Result<ExprType, String> {
    match name {
        "true" | "false" => Ok(ExprType::Bool),
        _ => Err(format!("undeclared variable '{}'", name)),
    }
}

fn check_expr(expr: &BoundedExpr) -> Result<TypedExpr, String> {
    match expr {
        BoundedExpr::Literal(value) => check_literal(value),
        BoundedExpr::TemplateLiteral(parts) => check_template(parts),
        BoundedExpr::UnresolvedName(name) => resolve_zero_arity_tag(name),
        BoundedExpr::Binary { op, left, right } => {
            let left_type = infer(left)?;
            let right_type = infer(right)?;
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

fn infer(expr: &BoundedExpr) -> Result<ExprType, String> {
    match expr {
        BoundedExpr::Literal(value) => infer_literal(value),
        BoundedExpr::TemplateLiteral(parts) => infer_template(parts),
        BoundedExpr::UnresolvedName(name) => infer_zero_arity_tag(name),
        BoundedExpr::Binary { left, right, .. } => {
            let left_type = infer(left)?;
            let right_type = infer(right)?;
            if !matches!((left_type, right_type), (ExprType::I32, ExprType::I32)) {
                return Err("binary operation operands must be of type i32".into());
            }
            Ok(ExprType::I32)
        }
        BoundedExpr::Var(_) => Ok(ExprType::I32),
        BoundedExpr::FuncCall(_) => Ok(ExprType::I32),
    }
}

fn check_template(parts: &[BoundedTemplatePart]) -> Result<TypedExpr, String> {
    let mut typed_parts = Vec::with_capacity(parts.len());
    for part in parts {
        match part {
            BoundedTemplatePart::Text(s) => typed_parts.push(TypedTemplatePart::Text(s.clone())),
            BoundedTemplatePart::Expr(e) => {
                let typed = check_expr(e)?;
                check_interpolatable(typed.ty())?;
                typed_parts.push(TypedTemplatePart::Expr(typed));
            }
        }
    }
    Ok(TypedExpr::Template(typed_parts))
}

fn infer_template(parts: &[BoundedTemplatePart]) -> Result<ExprType, String> {
    for part in parts {
        if let BoundedTemplatePart::Expr(e) = part {
            check_interpolatable(infer(e)?)?;
        }
    }
    Ok(ExprType::Str)
}

fn check_interpolatable(ty: ExprType) -> Result<(), String> {
    match ty {
        ExprType::I32 | ExprType::Bool | ExprType::Str | ExprType::Char => Ok(()),
        ExprType::Struct(name) => Err(format!(
            "cannot interpolate a value of type '{name}' into a literal - no string conversion is defined for it yet"
        )),
    }
}

fn infer_literal(value: &str) -> Result<ExprType, String> {
    if let Some(first) = value.chars().next() {
        match first {
            '"' => {
                validate_quoted_string(value)?;
                return Ok(ExprType::Str);
            }
            '`' => return Ok(ExprType::Str),
            '\'' => {
                validate_char_atom(value)?;
                return Ok(ExprType::Char);
            }
            _ => {}
        }
    }

    let (body, tag) = split_tag(value);
    match tag {
        None => body
            .parse::<i32>()
            .map(|_| ExprType::I32)
            .map_err(|_| format!("unsupported literal: '{value}'")),
        Some(tag_name) => {
            let parts: Vec<&str> = body.split(':').collect();
            let tag_def = lookup_tag(tag_name, parts.len())
                .ok_or_else(|| format!("no literal of shape {} field(s) tagged '{tag_name}' is defined (in '{value}')", parts.len()))?;
            for part in &parts {
                part.parse::<i32>()
                    .map_err(|_| format!("invalid field '{part}' in literal '{value}'"))?;
            }
            Ok(ExprType::Struct(tag_def.struct_name))
        }
    }
}

struct TagDef {
    struct_name: &'static str,
}

fn lookup_tag(tag: &str, field_count: usize) -> Option<TagDef> {
    match (tag, field_count) {
        ("R", 2) => Some(TagDef { struct_name: "Rectangle" }),
        ("R", 3) => Some(TagDef { struct_name: "Cuboid" }),
        _ => None,
    }
}

fn split_tag(text: &str) -> (&str, Option<&str>) {
    match text.find(|c: char| c.is_ascii_alphabetic()) {
        Some(idx) => (&text[..idx], Some(&text[idx..])),
        None => (text, None),
    }
}

fn check_literal(value: &str) -> Result<TypedExpr, String> {
    if let Some(first) = value.chars().next() {
        match first {
            '"' => {
                let decoded = unescape(strip_delimiters(value, '"'))?;
                return Ok(TypedExpr::Literal { value: decoded, ty: ExprType::Str });
            }
            '`' => {
                let decoded = strip_delimiters(value, '`').to_string();
                return Ok(TypedExpr::Literal { value: decoded, ty: ExprType::Str });
            }
            '\'' => {
                let decoded = unescape(strip_delimiters(value, '\''))?;
                if decoded.chars().count() != 1 {
                    return Err(format!("a char literal must contain exactly one character, found '{value}'"));
                }
                return Ok(TypedExpr::Literal { value: decoded, ty: ExprType::Char });
            }
            _ => {}
        }
    }

    let (body, tag) = split_tag(value);
    match tag {
        None => {
            body.parse::<i32>()
                .map_err(|_| format!("unsupported literal: '{value}'"))?;
            Ok(TypedExpr::Literal { value: value.to_string(), ty: ExprType::I32 })
        }
        Some(tag_name) => {
            let parts: Vec<&str> = body.split(':').collect();
            let tag_def = lookup_tag(tag_name, parts.len())
                .ok_or_else(|| format!("no literal of shape {} field(s) tagged '{tag_name}' is defined (in '{value}')", parts.len()))?;
            let mut fields = Vec::with_capacity(parts.len());
            for part in &parts {
                part.parse::<i32>()
                    .map_err(|_| format!("invalid field '{part}' in literal '{value}'"))?;
                fields.push(TypedExpr::Literal { value: part.to_string(), ty: ExprType::I32 });
            }
            Ok(TypedExpr::StructLiteral { name: tag_def.struct_name, fields })
        }
    }
}

fn validate_quoted_string(raw: &str) -> Result<(), String> {
    unescape(strip_delimiters(raw, '"')).map(|_| ())
}

fn validate_char_atom(raw: &str) -> Result<(), String> {
    let unescaped = unescape(strip_delimiters(raw, '\''))?;
    if unescaped.chars().count() != 1 {
        return Err(format!("a char literal must contain exactly one character, found '{raw}'"));
    }
    Ok(())
}

fn strip_delimiters(raw: &str, quote: char) -> &str {
    let triple: String = std::iter::repeat(quote).take(3).collect();
    if raw.starts_with(&triple) {
        &raw[3..raw.len() - 3]
    } else {
        &raw[1..raw.len() - 1]
    }
}

fn unescape(body: &str) -> Result<String, String> {
    let mut result = String::new();
    let mut chars = body.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            result.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => result.push('\n'),
            Some('t') => result.push('\t'),
            Some('r') => result.push('\r'),
            Some('\\') => result.push('\\'),
            Some('"') => result.push('"'),
            Some('\'') => result.push('\''),
            Some('`') => result.push('`'),
            Some('0') => result.push('\0'),
            Some(other) => return Err(format!("unknown escape sequence '\\{other}'")),
            None => return Err("dangling escape character at end of literal".to_string()),
        }
    }
    Ok(result)
}
