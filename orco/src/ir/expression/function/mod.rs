use super::*;

/// Function signature (Arguments and return type)
pub mod signature;
pub use signature::Signature;

/// A function
#[derive(Clone, Debug)]
pub struct Function {
    /// Function signature
    pub signature: Signature,
    /// Function body
    pub body: Expression,
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.signature, self.body)
    }
}
