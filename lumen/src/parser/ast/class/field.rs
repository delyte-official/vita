use std::fmt;

pub struct Field {
    pub name: String,
    pub ty: String,
    pub mutable: bool,
}

impl fmt::Debug for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}: {};",
            if self.mutable { "var" } else { "val" },
            self.name,
            self.ty
        )
    }
}