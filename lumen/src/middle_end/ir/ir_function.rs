use crate::middle_end::Instruction;

pub struct IRFunction {
    pub name: String,
    pub instructions: Vec<Instruction>,
    pub local_count: usize,
    pub temp_count: usize,
}
