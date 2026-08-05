use std::fmt;

use crate::binder::BoundedExpr;

pub enum BoundedStmt {
    Return(BoundedExpr),
    VarDecl(usize, String, Option<String>, BoundedExpr),
    ValDecl(usize, String, Option<String>, BoundedExpr),
    If {
        condition: BoundedExpr,
        then_branch: Vec<BoundedStmt>,
        elif_branches: Option<Vec<(BoundedExpr, Vec<BoundedStmt>)>>,
        else_branch: Option<Vec<BoundedStmt>>,
    },
    Assign(usize, String, BoundedExpr),
    FuncCall(usize, String),
}

impl BoundedStmt {
    pub fn fmt_with_indent(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        let spaces = "\t".repeat(indent);
        match self {
            BoundedStmt::Return(expr) => {
                write!(f, "{}return {:?};", spaces, expr)
            }
            BoundedStmt::VarDecl(index, name, type_, expr) => {
                let type_str = match type_ {
                    Some(t) => format!(": {}", t),
                    None => String::new(),
                };
                write!(f, "{}var {}#{}{} = {:?};", spaces, name, index, type_str, expr)
            }
            BoundedStmt::ValDecl(index, name, type_, expr) => {
                let type_str = match type_ {
                    Some(t) => format!(": {}", t),
                    None => String::new(),
                };
                write!(f, "{}val {}#{}{} = {:?};", spaces, name, index, type_str, expr)
            }
            BoundedStmt::If { condition, then_branch, elif_branches, else_branch } => {
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
            BoundedStmt::Assign(index, name, expr) => {
                write!(f, "{}{}#{} = {:?};", spaces, name, index, expr)
            }
            BoundedStmt::FuncCall(index, name) => {
                write!(f, "{}{}#{}();", spaces, name, index)
            }
        }
    }
}

impl fmt::Debug for BoundedStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_indent(f, 0)
    }
}