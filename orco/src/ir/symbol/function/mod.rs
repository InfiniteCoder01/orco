use super::*;

/// Function signature
pub mod signature;
pub use signature::Signature;

#[derive(Debug)]
/// A function
pub struct Function {
    /// Function signature
    pub signature: Signature,
    /// Function body
    pub body: std::sync::Mutex<Spanned<expression::Block>>,
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
    pub fn infer_and_check_types(&self, reporter: &mut dyn crate::diagnostics::ErrorReporter) {
        let mut type_inference =
            crate::type_inference::TypeInference::new(&self.signature.return_type, reporter);
        let mut body = self.body.lock().unwrap();
        body.infer_types(&mut type_inference);
        body.finish_and_check_types(&mut type_inference);
    }

    /// Format
    pub fn format(&self, f: &mut std::fmt::Formatter<'_>, name: Option<&str>) -> std::fmt::Result {
        self.signature.format(f, name)?;
        write!(f, " {}", self.body.lock().unwrap().inner)?;
        Ok(())
    }
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f, None)
    }
}
