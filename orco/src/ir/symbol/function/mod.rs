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
    pub body: std::sync::Mutex<expression::Expression>,
}

impl Function {
    /// Create a new function
    pub fn new(signature: Signature, body: expression::Expression) -> Self {
        Self {
            signature,
            body: body.into(),
        }
    }

    /// Infer types
    pub fn infer_and_check_types(
        &self,
        reporter: &mut dyn crate::diagnostics::ErrorReporter,
        root_module: &Module,
        current_module: &Module,
        current_module_path: &Path,
        symbol_resolver: &dyn Fn(&mut TypeInference, &Path) -> Option<SymbolReference>,
    ) {
        let mut type_inference = crate::type_inference::TypeInference::new(
            &self.signature.return_type,
            reporter,
            root_module,
            current_module,
            current_module_path,
            symbol_resolver,
        );

        type_inference.push_scope();
        for arg in self.signature.args.iter() {
            Expression::VariableDeclaration(arg.clone()).infer_types(&mut type_inference);
        }

        let mut body = self.body.lock().unwrap();
        body.infer_types(&mut type_inference);

        type_inference.pop_scope();
        println!("{}", body);

        let return_type = body.finish_and_check_types(&mut type_inference);
        if !return_type.morphs(&self.signature.return_type) {
            reporter.report_type_error(
                format!(
                    "Return type mismatch: expected '{}', got '{}'",
                    self.signature.return_type.inner, return_type
                ),
                body.span(),
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
        write!(f, "{} {}", self.signature, self.body.lock().unwrap())
    }
}
