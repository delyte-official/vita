use crate::type_checker::TypedFunction;

pub struct TypedProgram {
    pub functions: Vec<TypedFunction>,
}