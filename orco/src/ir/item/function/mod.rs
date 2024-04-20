use super::*;

/// Function signature
pub mod signature;
pub use signature::Signature;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
/// A function
pub struct Function {
    /// Function signature
    pub signature: Signature,
    /// Function body
    pub body: std::cell::RefCell<expression::Block>,
}

impl Function {
    /// Create a new function
    pub fn new(signature: Signature, body: expression::Block) -> Self {
        Self {
            signature,
            body: std::cell::RefCell::new(body),
        }
    }

    /// Infer types
    pub fn infer_types(&self, module: &Module) {
        let type_inference = crate::TypeInferenceInfo {
            module,
            return_type: &self.signature.return_type,
        };
        self.body
            .borrow_mut()
            .infer_types(&self.signature.return_type, &type_inference);
    }

    /// Format
    pub fn format(&self, f: &mut std::fmt::Formatter<'_>, name: Option<&str>) -> std::fmt::Result {
        self.signature.format(f, name)?;
        write!(f, " {}", self.body.borrow())?;
        Ok(())
    }
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f, None)
    }
}
