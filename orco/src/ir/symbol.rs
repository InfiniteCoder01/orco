use super::*;

#[derive(Clone, Debug)]
/// A symbol
pub struct Symbol {
    /// Symbol name
    pub name: Name,
    /// Symbol value
    pub value: Expression,
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "const {} = {};", self.name, self.value)
    }
}
