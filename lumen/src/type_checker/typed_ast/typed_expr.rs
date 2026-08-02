use crate::{parser::BinaryOp, type_checker::ExprType};

#[derive(Debug)]
pub enum TypedTemplatePart {
    Text(String),
    Expr(TypedExpr),
}

#[derive(Debug)]
pub enum TypedExpr {
    Literal {
        value: String,
        ty: ExprType,
    },
    StructLiteral {
        name: &'static str,
        fields: Vec<TypedExpr>,
    },
    Template(Vec<TypedTemplatePart>),
    Binary {
        op: BinaryOp,
        left: Box<TypedExpr>,
        right: Box<TypedExpr>,
        ty: ExprType,
    },
    Var(usize),
    FuncCall(usize),
}

impl TypedExpr {
    pub fn ty(&self) -> ExprType {
        match self {
            TypedExpr::Literal { ty, .. } => *ty,
            TypedExpr::StructLiteral { name, .. } => ExprType::Struct(name),
            TypedExpr::Template(_) => ExprType::Str,
            TypedExpr::Binary { ty, .. } => *ty,
            TypedExpr::Var(_) => ExprType::I32,
            TypedExpr::FuncCall(_) => ExprType::I32,
        }
    }
}
