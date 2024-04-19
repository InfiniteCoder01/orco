use super::*;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Function signature (i.e. parameters and return type)
pub struct Signature {
    /// Function return type
    pub return_type: Type,
}

impl Signature {
    /// Create a new function signature
    pub fn new(return_type: Type) -> Self {
        Self { return_type }
    }

    /// Format
    pub fn format(&self, f: &mut std::fmt::Formatter<'_>, name: Option<&str>) -> std::fmt::Result {
        match name {
            Some(name) => write!(f, "fn {}() -> {}", name, self.return_type),
            None => write!(f, "fn () -> {}", self.return_type),
        }
    }
}

impl std::fmt::Display for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f, None)
    }
}
