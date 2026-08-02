pub mod type_checker;
pub mod expr_type;
pub mod typed_ast;

pub use type_checker::check;
pub use expr_type::ExprType;
pub use typed_ast::*;