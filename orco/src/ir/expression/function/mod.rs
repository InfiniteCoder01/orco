use super::*;

/// Function signature (Arguments and return type)
pub mod signature;
pub use signature::Signature;

/// A function
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Function {
    /// Function signature
    pub signature: Signature,
    /// Function body
    pub body: std::sync::Mutex<Expression>,
    /// Span of the function
    #[derivative(Debug = "ignore")]
    pub span: Option<Span>,
    /// Metadata
    #[derivative(Debug = "ignore")]
    pub metadata: Box<dyn FunctionMetadata>,
}

impl Function {
    /// Create a new function
    pub fn new(
        signature: Signature,
        body: Expression,
        span: Option<Span>,
        metadata: impl FunctionMetadata + 'static,
    ) -> Self {
        Self {
            signature,
            body: body.into(),
            span,
            metadata: Box::new(metadata),
        }
    }

    /// Infer & check types
    pub fn infer_and_check_types(&self, type_inference: &mut TypeInference) {
        let old_return_type = type_inference
            .return_type
            .replace(self.signature.return_type.clone());
        let old_scopes = std::mem::take(&mut type_inference.scopes);

        type_inference.push_scope();
        for arg in self.signature.args.iter() {
            arg.as_ref().infer_types(type_inference);
        }

        let mut body = self.body.try_lock().unwrap();
        body.infer_types(type_inference);
        type_inference.pop_scope();

        for arg in self.signature.args.iter() {
            arg.finish_and_check_types(type_inference);
        }
        let return_type = body.finish_and_check_types(type_inference);
        if !return_type.morphs(&self.signature.return_type) {
            type_inference.report(self.metadata.return_type_mismatch(
                &self.signature.return_type,
                &return_type,
                body.span().cloned(),
            ));
        }

        type_inference.scopes = old_scopes;
        type_inference.return_type = old_return_type;
    }
}

impl Clone for Function {
    fn clone(&self) -> Self {
        Self {
            signature: self.signature.clone(),
            body: std::sync::Mutex::new(self.body.try_lock().unwrap().clone()),
            span: self.span.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fn {} {}", self.signature, self.body.try_lock().unwrap())
    }
}

/// Frontend metadata for a function
pub trait FunctionMetadata: Metadata {
    /// Return type mismatch error callback
    fn return_type_mismatch(
        &self,
        expected: &Spanned<Type>,
        got: &Type,
        expression_span: Option<Span>,
    ) -> Report {
        Report::build(ReportKind::Error)
            .with_code("typechecking::return_type_mismatch")
            .with_message(format!(
                "Return type mismatch: expected '{}', got '{}'",
                expected, got
            ))
            .opt_label(expression_span, |label| {
                label.with_message("Here").with_color(colors::Got)
            })
            .opt_label(expected.span.clone(), |label| {
                label
                    .with_message("Expected because of this")
                    .with_color(colors::Expected)
            })
            .finish()
    }
}

impl_metadata!(FunctionMetadata);

/// An extern function
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct ExternFunction {
    /// Extern function name
    pub name: Name,
    /// Function signature
    pub signature: Signature,
    /// Span of the extern function declaration
    #[derivative(Debug = "ignore")]
    pub span: Option<Span>,
    /// Metadata
    #[derivative(Debug = "ignore")]
    pub metadata: Box<dyn ExternFunctionMetadata>,
}

impl ExternFunction {
    /// Create a new extern function
    pub fn new(
        name: Name,
        signature: Signature,
        span: Option<Span>,
        metadata: impl ExternFunctionMetadata + 'static,
    ) -> Self {
        Self {
            name,
            signature,
            span,
            metadata: Box::new(metadata),
        }
    }
}

impl std::fmt::Display for ExternFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "extern fn {}{}", self.name, self.signature)
    }
}

/// Frontend metadata for an extern function
pub trait ExternFunctionMetadata: Metadata {}
impl_metadata!(ExternFunctionMetadata);
