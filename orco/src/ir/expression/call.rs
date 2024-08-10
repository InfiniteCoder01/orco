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
    pub span: Option<Span>,
    /// Metadata
    #[derivative(Debug = "ignore")]
    pub metadata: Box<dyn CallMetadata>,
}

impl CallExpression {
    /// Create a new call expression
    pub fn new(
        expression: Expression,
        args: Spanned<Vec<Expression>>,
        span: Option<Span>,
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
        let mut expr_type = self.expression.infer_types(type_inference);
        type_inference.inline(&mut expr_type);
        if let Type::FunctionPointer(signature_args, r#return) = expr_type {
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
                type_inference.report(self.metadata.argument_count_mismatch(
                    self.expression.as_ref(),
                    &self.args,
                    &signature_args,
                ));
            }
            for (arg, signature_arg) in std::iter::zip(&mut self.args.inner, &signature_args.inner)
            {
                let arg_type = arg.finish_and_check_types(type_inference);
                if !arg_type.morphs(signature_arg) {
                    type_inference.report(self.metadata.argument_type_mismatch(
                        self.expression.as_ref(),
                        &arg_type,
                        arg.span().cloned(),
                        signature_arg,
                    ));
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

/// Frontend metadata for a function call
pub trait CallMetadata: Metadata {
    /// Argument count mismatch error callback
    fn argument_count_mismatch(
        &self,
        callee: &Expression,
        args: &Spanned<Vec<Expression>>,
        signature_args: &Spanned<Vec<Spanned<Type>>>,
    ) -> Report {
        Report::build(ReportKind::Error)
            .with_code("typechecking::argument_count_mismatch")
            .with_message(format!(
                "Argument count mismatch: '{callee}' expects {} argument{}, but {} {} given",
                signature_args.inner.len(),
                if signature_args.inner.len() > 1 {
                    "s"
                } else {
                    ""
                },
                args.inner.len(),
                if args.inner.len() > 1 { "were" } else { "was" }
            ))
            .opt_label(args.span.clone(), |label| {
                label.with_message("Arguments").with_color(colors::Got)
            })
            .opt_label(signature_args.span.clone(), |label| {
                label
                    .with_message("Expected because of this")
                    .with_color(colors::Expected)
            })
            .finish()
    }

    /// Argument type mismatch error callback
    fn argument_type_mismatch(
        &self,
        expression: &Expression,
        arg: &Type,
        arg_span: Option<Span>,
        signature_arg: &Spanned<Type>,
    ) -> Report {
        Report::build(ReportKind::Error)
            .with_code("typechecking::argument_type_mismatch")
            .with_message(format!(
                "Incompatible argument types for '{expression}': expected '{}', got '{arg}'",
                signature_arg.inner
            ))
            .opt_label(arg_span, |label| {
                label.with_message("Argument").with_color(colors::Got)
            })
            .opt_label(signature_arg.span.clone(), |label| {
                label
                    .with_message("Expected because of this")
                    .with_color(colors::Expected)
            })
            .finish()
    }
}
impl_metadata!(CallMetadata);
