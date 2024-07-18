use super::*;

/// Block expression, contains multiple expressions (something along { expr1; expr2; })
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct Block {
    /// Block content
    pub expressions: Vec<Expression>,
    /// What this block evaluates to (basically tail expression)
    pub tail_expression: Option<Box<Expression>>,
    /// Span of the expression
    pub span: Span,
    /// Set to true, if the block does not form a new scope
    pub transparent: bool,
    /// Metadata
    #[derivative(Debug = "ignore", Default(value = "Box::new(())"))]
    pub metadata: Box<dyn BlockMetadata>,
}

impl Block {
    /// Create a new block
    pub fn new(
        expressions: Vec<Expression>,
        tail_expression: Option<Box<Expression>>,
        span: Span,
        transparent: bool,
        metadata: impl BlockMetadata + 'static,
    ) -> Self {
        Self {
            expressions,
            tail_expression,
            span,
            transparent,
            metadata: Box::new(metadata),
        }
    }

    /// Get the type this block evaluates to
    pub fn get_type(&self) -> Type {
        for expression in &self.expressions {
            if expression.get_type() == Type::Never {
                return Type::Never;
            }
        }
        self.tail_expression
            .as_ref()
            .map_or_else(Type::unit, |expr| expr.get_type())
    }

    /// Infer types
    pub fn infer_types(&mut self, type_inference: &mut TypeInference) -> Type {
        if !self.transparent {
            type_inference.push_scope();
        }
        let mut r#type = Type::unit();
        for expression in &mut self.expressions {
            let expr_type = expression.infer_types(type_inference);
            if expr_type == Type::Never {
                r#type = Type::Never;
            }
        }
        if let Some(expression) = self.tail_expression.as_mut() {
            let expr_type = expression.infer_types(type_inference);
            if r#type != Type::Never {
                r#type = expr_type;
            }
        }
        if !self.transparent {
            type_inference.pop_scope();
        }
        r#type
    }

    /// Finish and check types
    pub fn finish_and_check_types(&mut self, type_inference: &mut TypeInference) -> Type {
        let mut r#type = Type::Unit;
        let mut unreachable_span: Option<Span> = None;
        for expression in &mut self.expressions {
            if r#type == Type::Never {
                let span = expression.span();
                unreachable_span.get_or_insert(span).1.end = span.1.end;
            }
            let expr_type = expression.finish_and_check_types(type_inference);
            if expr_type == Type::Never {
                r#type = Type::Never;
            }
        }

        if let Some(expression) = &mut self.tail_expression {
            let expr_type = expression.finish_and_check_types(type_inference);
            if r#type == Type::Never {
                let span = expression.span();
                unreachable_span.get_or_insert(span).1.end = span.1.end;
            } else {
                r#type = expr_type;
            }
        }

        if let Some(span) = unreachable_span {
            self.metadata.unreachable_code(
                type_inference,
                UnreachableCode {
                    src: span.named_source(),
                    span: span.source_span(),
                },
            );
        }

        r#type
    }
}

#[derive(Error, Debug, Diagnostic)]
#[error("Unreachable code")]
#[diagnostic(code(potential_bugs::unreachable_code), severity(Warning))]
/// Unreachable code
pub struct UnreachableCode {
    #[source_code]
    /// File where the error occurred
    pub src: NamedSource<Src>,
    #[label("This code is unreachable")]
    /// Span of the unreachable code
    pub span: SourceSpan,
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{")?;
        for expression in &self.expressions {
            write!(f, "{}", indent::indent_all_by(4, format!("{expression}")))?;
            if !expression.is_block() {
                write!(f, ";")?;
            }
            writeln!(f)?;
        }
        if let Some(expression) = &self.tail_expression {
            writeln!(
                f,
                "{} // Tail expression",
                indent::indent_all_by(4, format!("{expression}"))
            )?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

declare_metadata! {
    /// Frontend metadata for block expression
    trait BlockMetadata {
        Diagnostics:
        /// Callback of unreachable code warning
        unreachable_code(UnreachableCode)
    }
}
