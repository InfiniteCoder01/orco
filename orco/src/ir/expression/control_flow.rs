use super::*;

/// Return expression
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct ReturnExpression {
    /// Expression to return
    pub expression: Box<Expression>,
    /// Span of the expression
    #[derivative(Debug = "ignore")]
    pub span: Option<Span>,
    /// Metadata
    #[derivative(Debug = "ignore")]
    pub metadata: Box<dyn ReturnMetadata>,
}

impl ReturnExpression {
    /// Create a new return expression
    pub fn new(
        expression: Box<Expression>,
        span: Option<Span>,
        metadata: impl ReturnMetadata + 'static,
    ) -> Self {
        Self {
            expression,
            span,
            metadata: Box::new(metadata),
        }
    }

    /// Get the type this exprssion evaluates to
    pub fn get_type(&self) -> Type {
        Type::Never
    }

    /// Infer the type for this expression
    pub fn infer_types(&mut self, type_inference: &mut TypeInference) -> Type {
        let r#type = self.expression.infer_types(type_inference);
        if let Some(return_type) = type_inference.return_type.take() {
            type_inference.equate(&r#type, &return_type);
            type_inference.return_type = Some(return_type);
        }
        Type::Never
    }

    /// Finish and check types
    pub fn finish_and_check_types(&mut self, type_inference: &mut TypeInference) -> Type {
        let r#type = self.expression.finish_and_check_types(type_inference);
        if let Some(return_type) = &type_inference.return_type {
            if !r#type.morphs(return_type) {
                type_inference.report(self.metadata.return_type_mismatch(
                    &r#type,
                    self.expression.span().cloned(),
                    return_type,
                ));
            }
        } else {
            todo!("return without a function error message")
        }
        Type::Never
    }
}

impl std::fmt::Display for ReturnExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "return {}", self.expression)
    }
}

/// Frontend metadata for the return expression
pub trait ReturnMetadata: Metadata {
    /// Return type mismatch error callback
    fn return_type_mismatch(
        &self,
        r#type: &Type,
        span: Option<Span>,
        signature_type: &Spanned<Type>,
    ) -> Report {
        Report::build(ReportKind::Error)
            .with_code("typechecking::return_type_mismatch")
            .with_message(format!(
                "Return type mismatch: expected '{}', got '{}'",
                signature_type, r#type
            ))
            .opt_label(span, |label| {
                label.with_message("Here").with_color(colors::Got)
            })
            .opt_label(signature_type.span.clone(), |label| {
                label
                    .with_message("Expected because of this")
                    .with_color(colors::Expected)
            })
            .finish()
    }
}
impl_metadata!(ReturnMetadata);
