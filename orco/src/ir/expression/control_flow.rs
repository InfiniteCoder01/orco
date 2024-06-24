use super::*;

/// Return expression
#[derive(Debug)]
pub struct ReturnExpression(pub Box<Expression>);

impl ReturnExpression {
    /// Get the type this exprssion evaluates to
    pub fn get_type(&self) -> Type {
        Type::Never
    }

    /// Infer the type for this expression
    pub fn infer_types(&mut self, type_inference: &mut TypeInference) -> Type {
        let r#type = self.0.infer_types(type_inference);
        type_inference.equate(&r#type, type_inference.return_type);
        Type::Never
    }

    /// Finish and check types
    pub fn finish_and_check_types(
        &mut self,
        span: Span,
        type_inference: &mut TypeInference,
    ) -> Type {
        let r#type = self.0.finish_and_check_types(type_inference);
        if !r#type.morphs(type_inference.return_type) {
            type_inference.reporter.report_type_error(
                format!(
                    "Return type mismatch: expected '{}', got '{}'",
                    type_inference.return_type.inner, r#type
                ),
                span,
                vec![(
                    "Expected because of this",
                    type_inference.return_type.span.clone(),
                )],
            );
        }
        Type::Never
    }
}

impl std::fmt::Display for ReturnExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "return {}", self.0)
    }
}
