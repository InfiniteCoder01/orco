use super::*;

/// Function signature
pub mod signature;
pub use signature::Signature;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// A function
pub struct Function {
    /// Function signature
    pub signature: Signature,
    /// Function body
    pub body: expression::Block,
}

impl Function {
    /// Create a new function
    pub fn new(signature: Signature, body: expression::Block) -> Self {
        Self { signature, body }
    }

    /// Infer types
    pub fn infer_types(&mut self) {
        self.body
            .infer_types(&self.signature.return_type, &self.signature.return_type);
    }

    /// Format
    pub fn format(&self, f: &mut std::fmt::Formatter<'_>, name: Option<&str>) -> std::fmt::Result {
        self.signature.format(f, name)?;
        write!(f, " {}", self.body)?;
        Ok(())
    }
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f, None)
    }
}
