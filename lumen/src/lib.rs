pub mod backend;
pub mod binder;
pub mod lexer;
pub mod middle_end;
pub mod parser;
pub mod typechecker;

use std::path::Path;

pub fn compile(source: &str, output_path: &Path) -> Result<(), String> {
    let ast = parser::parse(source)?;
    let bound = binder::bind(ast)?;
    let typed = typechecker::check(bound)?;
    let ir = middle_end::lower(typed);
    backend::compile_to_executable(&ir, output_path)?;
    Ok(())
}
