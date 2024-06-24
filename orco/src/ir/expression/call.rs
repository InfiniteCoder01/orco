use super::*;

/// Call expression (function call)
#[derive(Debug)]
pub struct CallExpression {
    /// Expression to call ([SymbolReference] to call a function)
    pub expression: Box<Expression>,
    /// Arguments to the call
    pub args: Spanned<Vec<Expression>>,
}

impl CallExpression {
    /// Create a new call expression
    pub fn new(expression: Expression, args: Spanned<Vec<Expression>>) -> Self {
        Self {
            expression: Box::new(expression),
            args,
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
            Type::Error
        }
    }

    /// Finish and check types
    pub fn finish_and_check_types(&mut self, type_inference: &mut TypeInference) -> Type {
        let r#type = self.expression.finish_and_check_types(type_inference);
        if let Type::FunctionPointer(signature_args, r#return) = r#type {
            if self.args.inner.len() != signature_args.len() {
                type_inference.reporter.report_type_error(
                            format!(
                                "Argument count mismatch: Function '{}' expects {} arguments, but {} were given",
                                self.expression,
                                signature_args.len(),
                                self.args.inner.len()
                            ),
                            self.args.span.clone(),
                            vec![("Expected because of this", signature_args.span.clone())], 
                        );
            }
            for (arg, signature_arg) in std::iter::zip(&mut self.args.inner, &signature_args.inner)
            {
                let arg_type = arg.finish_and_check_types(type_inference);
                if !arg_type.morphs(signature_arg) {
                    type_inference.reporter.report_type_error(
                                format!(
                                    "Incompatible argument types for function '{}': expected '{}', got '{}'",
                                    self.expression,
                                    arg_type, signature_arg.inner
                                ),
                                arg.span(),
                                vec![("Expected because of this", signature_arg.span.clone())],
                            );
                }
            }
            r#return.inner.clone()
        } else {
            if r#type != Type::Error {
                type_inference.reporter.report_type_error(
                    format!("Can't call {}", r#type),
                    self.expression.span(),
                    vec![],
                );
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
