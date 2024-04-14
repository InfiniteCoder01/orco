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
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// A function
pub struct Function {
    /// Function signature
    pub signature: Signature,
}

impl Function {
    /// Create a new function
    pub fn new(signature: Signature) -> Self {
        Self { signature }
    }
}
