use crate::middle_end::Value;

pub enum Instruction {
    Add(usize, Value, Value),
    Sub(usize, Value, Value),
    Mul(usize, Value, Value),
    Div(usize, Value, Value),
    VarDecl(usize, Value),
    Return(Value),
    Call(usize),
}