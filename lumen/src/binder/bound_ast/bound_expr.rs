use crate::parser::BinaryOp;

#[derive(Debug)]
pub enum BoundedTemplatePart {
    Text(String),
    Expr(Box<BoundedExpr>),
}

#[derive(Debug)]
pub enum BoundedExpr {
    Literal(String),
    TemplateLiteral(Vec<BoundedTemplatePart>),
    UnresolvedName(String),
    Binary {
        op: BinaryOp,
        left: Box<BoundedExpr>,
        right: Box<BoundedExpr>,
    },
    Var(usize),
    FuncCall(usize),
}
