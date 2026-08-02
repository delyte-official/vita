#[derive(Clone, Copy)]
pub enum Value {
    Const(i64),
    Temp(usize),
    Var(usize),
}