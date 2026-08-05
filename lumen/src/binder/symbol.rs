#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol {
    pub name: String,
    pub slot: usize,
    pub scope_depth: usize,
    pub mutable: bool,
}