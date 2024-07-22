use super::*;

/// Return expression
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct ReturnExpression(
    pub Box<Expression>,
    pub Span,
    #[derivative(Debug = "ignore")] pub Box<dyn ReturnMetadata>,
);

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
    pub fn finish_and_check_types(&mut self, type_inference: &mut TypeInference) -> Type {
        let r#type = self.0.finish_and_check_types(type_inference);
        if !r#type.morphs(type_inference.return_type) {
            self.2.return_type_mismatch(
                type_inference,
                ReturnTypeMismatch {
                    expected: type_inference.return_type.inner.clone(),
                    got: r#type,
                    src: self.0.span().named_source(),
                    expression_span: self.0.span().source_span(),
                    return_type_span: type_inference.return_type.span.source_span(),
                },
            );
        }
        Type::Never
    }
}

#[derive(Error, Debug, Diagnostic)]
#[error("Return type mismatch: expected '{expected}', got '{got}'")]
#[diagnostic(code(typechecking::return_type_mismatch))]
/// Return type mismatch
pub struct ReturnTypeMismatch {
    /// Expected return type
    pub expected: Type,
    /// Got type
    pub got: Type,

    #[source_code]
    /// File where the error occurred
    pub src: NamedSource<Src>,
    #[label("Here")]
    /// Span of the expression returned
    pub expression_span: SourceSpan,
    #[label("Expected because of this")]
    /// Span of the return type
    pub return_type_span: SourceSpan,
}

impl std::fmt::Display for ReturnExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "return {}", self.0)
    }
}

declare_metadata! {
    /// Frontend metadata for the return expression
    trait ReturnMetadata {
        Diagnostics:
        /// Return type mismatch error callback
        return_type_mismatch(ReturnTypeMismatch)
    }
}
