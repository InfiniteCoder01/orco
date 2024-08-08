use super::*;

/// Call expression (function call)
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct CallExpression {
    /// Expression to call ([SymbolReference] to call a function)
    pub expression: Box<Expression>,
    /// Arguments to the call
    pub args: Spanned<Vec<Expression>>,
    /// Span of the expression
    #[derivative(Debug = "ignore")]
    pub span: Span,
    /// Metadata
    #[derivative(Debug = "ignore")]
    pub metadata: Box<dyn CallMetadata>,
}

impl CallExpression {
    /// Create a new call expression
    pub fn new(
        expression: Expression,
        args: Spanned<Vec<Expression>>,
        span: Span,
        metadata: impl CallMetadata + 'static,
    ) -> Self {
        Self {
            expression: Box::new(expression),
            args,
            span,
            metadata: Box::new(metadata),
        }
    }

    /// Get the type this call expression evaluates to
    pub fn get_type(&self) -> Type {
        match self.expression.get_type() {
            Type::FunctionPointer(_, r#return) => r#return.inner,
            _ => Type::Error,
        }
    }

    /// Infer the type of this call expression
    pub fn infer_types(&mut self, type_inference: &mut TypeInference) -> Type {
        let expr_type = self.expression.infer_types(type_inference);
        if let Type::FunctionPointer(signature_args, r#return) = type_inference.inline(expr_type) {
            for (arg, signature_arg) in std::iter::zip(&mut self.args.inner, signature_args.inner) {
                let arg_type = arg.infer_types(type_inference);
                type_inference.equate(&arg_type, &signature_arg.inner);
            }
            r#return.inner.clone()
        } else {
            Type::Wildcard
        }
    }

    /// Finish and check types
    pub fn finish_and_check_types(&mut self, type_inference: &mut TypeInference) -> Type {
        let r#type = self.expression.finish_and_check_types(type_inference);
        if let Type::FunctionPointer(signature_args, r#return) = r#type {
            if self.args.len() != signature_args.len() {
                self.metadata.argument_count_mismatch(
                    type_inference,
                    ArgumentCountMismatch {
                        expression: self.expression.as_ref().clone(),
                        expected: signature_args.len(),
                        given: self.args.len(),
                        src: self.args.span.as_ref().unwrap().named_source(),
                        args_span: self.args.span.as_ref().unwrap().source_span(),
                        signature_span: signature_args.span.as_ref().unwrap().source_span(),
                    },
                );
            }
            for (arg, signature_arg) in std::iter::zip(&mut self.args.inner, &signature_args.inner)
            {
                let arg_type = arg.finish_and_check_types(type_inference);
                if !arg_type.morphs(signature_arg) {
                    self.metadata.argument_type_mismatch(
                        type_inference,
                        ArgumentTypeMismatch {
                            expression: self.expression.as_ref().clone(),
                            expected: signature_arg.inner.clone(),
                            got: arg_type,
                            src: arg.span().as_ref().unwrap().named_source(),
                            arg_span: arg.span().as_ref().unwrap().source_span(),
                            signature_span: signature_arg.span.as_ref().unwrap().source_span(),
                        },
                    );
                }
            }
            r#return.inner.clone()
        } else {
            if r#type != Type::Error {
                todo!("Can't call error");
                // type_inference.abort_compilation = true;
            }
            Type::Error
        }
    }
}

#[derive(Error, Debug, Diagnostic)]
#[error("Argument count mismatch: Function '{expression}' expects {expected} argument(s), but {given} were(was) given")]
#[diagnostic(code(typechecking::call::argument_count_mismatch))]
/// Argument count mismatch
pub struct ArgumentCountMismatch {
    /// Expression
    pub expression: Expression,
    /// Number of expected arguments
    pub expected: usize,
    /// Number of given arguments
    pub given: usize,

    #[source_code]
    /// File where the error occurred
    pub src: NamedSource<Src>,
    #[label("Here")]
    /// Span of the args
    pub args_span: SourceSpan,
    #[label("Expected because of this")]
    /// Span of the signature
    pub signature_span: SourceSpan,
}

#[derive(Error, Debug, Diagnostic)]
#[error(
    "Incompatible argument types for function '{expression}': expected '{expected}', got '{got}'"
)]
#[diagnostic(code(typechecking::call::argument_type_mismatch))]
/// Argument type mismatch
pub struct ArgumentTypeMismatch {
    /// Expression
    pub expression: Expression,
    /// Expected type
    pub expected: Type,
    /// Got type
    pub got: Type,

    #[source_code]
    /// File where the error occurred
    pub src: NamedSource<Src>,
    #[label("Here")]
    /// Span of the argument
    pub arg_span: SourceSpan,
    #[label("Expected because of this")]
    /// Span of the argument type in signature
    pub signature_span: SourceSpan,
}

impl std::fmt::Display for CallExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(", self.expression)?;
        for (index, arg) in self.args.iter().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", arg)?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

declare_metadata! {
    /// Frontend metadata for a function call
    trait CallMetadata {
        Diagnostics:
        /// Argument count mismatch error callback
        argument_count_mismatch(ArgumentCountMismatch) abort_compilation;
        /// Argument type mismatch error callback
        argument_type_mismatch(ArgumentTypeMismatch) abort_compilation;
    }
}
