use std::fmt;

pub enum LiteralAtom {
    TypeAtom { name: String, ty: String },
    Text(String),
}

pub struct LiteralRepresentation {
    pub parts: Vec<LiteralAtom>,
}

impl fmt::Debug for LiteralRepresentation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let repr = self.parts.iter().map(|atom| {
            match atom {
                LiteralAtom::TypeAtom { name, ty } => format!("<{}: {}>", name, ty),
                LiteralAtom::Text(text) => text.clone(),
            }
        }).collect::<Vec<String>>().join(" ");
        write!(f, "{}", repr)
    }
}