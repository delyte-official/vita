use crate::middle_end::Instruction;
use crate::type_checker::ExprType;

pub struct IRFunction {
    pub name: String,
    pub instructions: Vec<Instruction>,
    pub local_count: usize,
    pub temp_count: usize,
    pub return_type: ExprType,
}
