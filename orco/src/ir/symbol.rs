use super::*;

#[derive(Debug)]
/// A symbol
pub struct Symbol {
    /// Symbol name
    pub name: Name,
    /// Symbol value
    pub value: Expression,
    /// Evaluated
    pub evaluated: Option<Value>,
}

impl Symbol {
    /// Create a new symbol
    pub fn new(name: Name, value: Expression) -> Self {
        Self {
            name,
            value,
            evaluated: None,
        }
    }
}

impl Clone for Symbol {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            value: self.value.clone(),
            evaluated: None,
        }
    }
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "comptime {} = {};", self.name, self.value)
    }
}
