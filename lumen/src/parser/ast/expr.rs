use super::BinaryOp;

#[derive(Debug)]
pub enum LiteralPart {
    Text(String),
    Expr(Expr),
}

#[derive(Debug)]
pub enum Expr {
    Literal(String),
    TemplateLiteral(Vec<LiteralPart>),
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Var(String),
    FuncCall { name: String },
}

impl Expr {
    pub fn binary(op: BinaryOp, left: Expr, right: Expr) -> Expr {
        Expr::Binary { op, left: Box::new(left), right: Box::new(right) }
    }
}
