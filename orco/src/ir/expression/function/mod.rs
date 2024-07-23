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

impl Function {
    /// Infer & check types
    pub fn infer_and_check_types(&mut self, type_inference: &mut TypeInference) {
        // TODO: Return type

        // type_inference.push_scope();
        // for arg in self.signature.args.iter() {
        //     Expression::VariableDeclaration(arg.clone()).infer_types(&mut type_inference);
        // }

        self.body.infer_types(type_inference);
        // type_inference.pop_scope();

        let return_type = self.body.finish_and_check_types(type_inference);
        if !return_type.morphs(&self.signature.return_type) {
            type_inference.reporter.report_type_error(
                format!(
                    "Return type mismatch: expected '{}', got '{}'",
                    self.signature.return_type.inner, return_type
                ),
                self.body.span(),
                vec![(
                    "Expected because of this",
                    self.signature.return_type.span.clone(),
                )],
            );
        }
    }
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.signature, self.body)
    }
}
