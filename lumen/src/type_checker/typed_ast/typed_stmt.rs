use std::fmt;

use crate::type_checker::TypedExpr;

pub enum TypedStmt {
    Return(TypedExpr),
    VarDecl(usize, String, TypedExpr),
    ValDecl(usize, String, TypedExpr),
    If {
        condition: TypedExpr,
        then_branch: Vec<TypedStmt>,
        elif_branches: Option<Vec<(TypedExpr, Vec<TypedStmt>)>>,
        else_branch: Option<Vec<TypedStmt>>,
    },
    Assign(usize, String, TypedExpr),
    FuncCall(usize, String),
}

impl TypedStmt {
    pub fn fmt_with_indent(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        let spaces = "\t".repeat(indent);
        match self {
            TypedStmt::Return(expr) => {
                write!(f, "{}return {:?};", spaces, expr)
            }
            TypedStmt::VarDecl(index, name, expr) => {
                write!(f, "{}var {}#{}: {:?} = {:?};", spaces, name, index, expr.ty(), expr)
            }
            TypedStmt::ValDecl(index, name, expr) => {
                write!(f, "{}val {}#{}: {:?} = {:?};", spaces, name, index, expr.ty(), expr)
            }
            TypedStmt::If { condition, then_branch, elif_branches, else_branch } => {
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
            TypedStmt::Assign(index, name, expr) => {
                write!(f, "{}{}#{} = {:?};", spaces, name, index, expr)
            }
            TypedStmt::FuncCall(index, name) => {
                write!(f, "{}{}#{}();", spaces, name, index)
            }
        }
    }
}

impl fmt::Debug for TypedStmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_with_indent(f, 0)
    }
}