use super::*;

/// Function signature
pub mod signature;
pub use signature::Signature;

#[derive(Clone, Debug, PartialEq, Eq)]
/// A function
pub struct Function {
    /// Function signature
    pub signature: Signature,
    /// Function body
    pub body: std::cell::RefCell<Spanned<expression::Block>>,
}

impl Function {
    /// Create a new function
    pub fn new(signature: Signature, body: Spanned<expression::Block>) -> Self {
        Self {
            signature,
            body: body.into(),
        }
    }

    /// Infer types
    pub fn infer_and_check_types(
        &self,
        root: &Module,
        reporter: &mut dyn crate::diagnostics::ErrorReporter,
    ) {
        let mut type_inference =
            crate::TypeInference::new(root, &self.signature.return_type, reporter);
        self.body
            .borrow_mut()
            .infer_types(&self.signature.return_type, &mut type_inference);
        self.body
            .borrow_mut()
            .finish_and_check_types(&mut type_inference);
    }

    /// Format
    pub fn format(&self, f: &mut std::fmt::Formatter<'_>, name: Option<&str>) -> std::fmt::Result {
        self.signature.format(f, name)?;
        write!(f, " {}", self.body.borrow().inner)?;
        Ok(())
    }
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f, None)
    }
}
