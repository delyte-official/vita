use std::fmt;

use super::Expr;

pub enum Stmt {
    Return(Expr),
    VarDecl(String, Option<String>, Expr),
    ValDecl(String, Option<String>, Expr),
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        elif_branches: Option<Vec<(Expr, Vec<Stmt>)>>,
        else_branch: Option<Vec<Stmt>>,
    },
    Assign(String, Expr),
    FuncCall{ name: String },
}

impl Stmt {
    pub fn fmt_with_indent(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        let spaces = "\t".repeat(indent);
        match self {
            Stmt::Return(expr) => {
                write!(f, "{}return {:?};", spaces, expr)
            }
            Stmt::VarDecl(name, type_, expr) => {
                let type_str = match type_ {
                    Some(t) => format!(": {}", t),
                    None => String::new(),
                };
                write!(f, "{}var {}{} = {:?};", spaces, name, type_str, expr)
            }
            Stmt::ValDecl(name, type_, expr) => {
                let type_str = match type_ {
                    Some(t) => format!(": {}", t),
                    None => String::new(),
                };
                write!(f, "{}val {}{} = {:?};", spaces, name, type_str, expr)
            }
            Stmt::If { condition, then_branch, elif_branches, else_branch } => {
                write!(f, "{}if ({:?}) {{\n", spaces, condition)?;
                
                for stmt in then_branch {
                    stmt.fmt_with_indent(f, indent + 1)?;
                    writeln!(f)?;
                }
                write!(f, "{}}}", spaces)?;

                if let Some(elif_list) = elif_branches {
                    for (cond, branch) in elif_list {
                        write!(f, " elif ({:?}) {{\n", cond)?;
                        for stmt in branch {
                            stmt.fmt_with_indent(f, indent + 1)?;
                            writeln!(f)?;
                        }
                        write!(f, "{}}}", spaces)?;
                    }
                }

                if let Some(else_list) = else_branch {
                    write!(f, " else {{\n")?;
                    for stmt in else_list {
                        stmt.fmt_with_indent(f, indent + 1)?;
                        writeln!(f)?;
                    }
                    write!(f, "{}}}", spaces)?;
                }
                write!(f, "")
            }
            Stmt::Assign(name, expr) => {
                write!(f, "{}{} = {:?};", spaces, name, expr)
            }
            Stmt::FuncCall { name } => {
                write!(f, "{}{}();", spaces, name)
            }
        }
    }
}

impl fmt::Debug for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_indent(f, 0)
    }
}