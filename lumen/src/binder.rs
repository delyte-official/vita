use crate::parser::{Program, Stmt};

pub struct BoundProgram {
    pub main_name: String,
    pub body: Vec<Stmt>,
}

pub fn bind(program: Program) -> Result<BoundProgram, String> {
    let known_names = vec![program.main.name.clone()];
    let _ = known_names;

    Ok(BoundProgram {
        main_name: program.main.name,
        body: program.main.body,
    })
}
