use crate::parser::*;

pub fn pretty_print(program: &Program) -> String {
    let mut out = String::new();
    for (i, func) in program.functions.iter().enumerate() {
        if i > 0 {
            out.push_str("\n\n");
        }
        print_function(func, &mut out);
    }
    out
}

fn print_function(func: &Function, out: &mut String) {
    out.push_str(&func.name);
    out.push_str(" {\n");
    print_stmts(&func.body, 1, out);
    out.push_str("}");
}

fn print_stmts(stmts: &[Stmt], indent: usize, out: &mut String) {
    for stmt in stmts {
        print_stmt(stmt, indent, out);
    }
}

fn indent_str(indent: usize) -> String {
    "    ".repeat(indent)
}

fn print_decl(keyword: &str, name: &str, ty: &Option<String>, expr: &Expr, pad: &str, out: &mut String) {
    out.push_str(pad);
    out.push_str(keyword);
    out.push(' ');
    out.push_str(name);
    if let Some(ty) = ty {
        out.push_str(": ");
        out.push_str(ty);
    }
    out.push_str(" = ");
    out.push_str(&print_expr(expr));
    out.push_str(";\n");
}

fn print_stmt(stmt: &Stmt, indent: usize, out: &mut String) {
    let pad = indent_str(indent);
    match stmt {
        Stmt::Return(expr) => {
            out.push_str(&pad);
            out.push_str("return ");
            out.push_str(&print_expr(expr));
            out.push_str(";\n");
        }
        Stmt::VarDecl(name, ty, expr) => print_decl("var", name, ty, expr, &pad, out),
        Stmt::ValDecl(name, ty, expr) => print_decl("val", name, ty, expr, &pad, out),
        Stmt::If { condition, then_branch, elif_branches, else_branch } => {
            out.push_str(&pad);
            out.push_str("if (");
            out.push_str(&print_expr(condition));
            out.push_str(") {\n");
            print_stmts(then_branch, indent + 1, out);
            out.push_str(&pad);
            out.push('}');

            if let Some(elifs) = elif_branches {
                for (cond, body) in elifs {
                    out.push_str(" elif (");
                    out.push_str(&print_expr(cond));
                    out.push_str(") {\n");
                    print_stmts(body, indent + 1, out);
                    out.push_str(&pad);
                    out.push('}');
                }
            }

            if let Some(else_body) = else_branch {
                out.push_str(" else {\n");
                print_stmts(else_body, indent + 1, out);
                out.push_str(&pad);
                out.push('}');
            }

            out.push('\n');
        }
        Stmt::Assign(name, expr) => {
            out.push_str(&pad);
            out.push_str(name);
            out.push_str(" = ");
            out.push_str(&print_expr(expr));
            out.push_str(";\n");
        }
        Stmt::FuncCall { name } => {
            out.push_str(&pad);
            out.push_str(name);
            out.push_str("();\n");
        }
    }
}

fn print_expr(expr: &Expr) -> String {
    print_expr_prec(expr, 0)
}

fn print_expr_prec(expr: &Expr, min_prec: u8) -> String {
    match expr {
        Expr::Literal(text) => text.clone(),
        Expr::TemplateLiteral(parts) => {
            let mut result = String::new();
            for part in parts {
                match part {
                    LiteralPart::Text(text) => result.push_str(text),
                    LiteralPart::Expr(expr) => {
                        result.push('{');
                        result.push_str(&print_expr(expr));
                        result.push('}');
                    }
                }
            }
            result
        }
        Expr::Var(name) => name.clone(),
        Expr::FuncCall { name } => format!("{name}()"),
        Expr::Binary { op, left, right } => {
            let prec = op.precedence();
            let left_str = print_expr_prec(left, prec);
            let right_str = print_expr_prec(right, op.minimum_binding_power());
            let inner = format!("{left_str} {} {right_str}", op_symbol(*op));
            if prec < min_prec {
                format!("({inner})")
            } else {
                inner
            }
        }
    }
}

fn op_symbol(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Plus => "+",
        BinaryOp::Minus => "-",
        BinaryOp::Star => "*",
        BinaryOp::Slash => "/",
    }
}
