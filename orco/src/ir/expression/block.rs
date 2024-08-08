use super::*;

/// Block expression, contains multiple expressions (something along { expr1; expr2; })
#[derive(Derivative, Clone)]
#[derivative(Debug, Default)]
pub struct Block {
    /// Block content
    pub expressions: Vec<Expression>,
    /// What this block evaluates to (basically tail expression)
    pub tail_expression: Option<Box<Expression>>,
    /// Set to true, if the block does not form a new scope
    pub transparent: bool,
    /// Span of the expression
    #[derivative(Debug = "ignore")]
    pub span: Option<Span>,
    /// Metadata
    #[derivative(Debug = "ignore", Default(value = "Box::new(())"))]
    pub metadata: Box<dyn BlockMetadata>,
}

impl Block {
    /// Create a new block
    pub fn new(
        expressions: Vec<Expression>,
        tail_expression: Option<Box<Expression>>,
        transparent: bool,
        span: Option<Span>,
        metadata: impl BlockMetadata + 'static,
    ) -> Self {
        Self {
            expressions,
            tail_expression,
            transparent,
            span,
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
        let mut unreachable_code: Option<UnreachableCode> = None;

        for expression in &mut self.expressions {
            if let Some(unreachable_code) = &mut unreachable_code {
                unreachable_code.extend(expression.span());
            }
            let expr_type = expression.finish_and_check_types(type_inference);
            if expr_type == Type::Never {
                r#type = Type::Never;
                unreachable_code = Some(UnreachableCode {
                    there_is_some: false,
                    unreachable_code: None,
                    reason: expression.span().cloned(),
                });
            }
        }

        if let Some(expression) = &mut self.tail_expression {
            let expr_type = expression.finish_and_check_types(type_inference);
            if let Some(unreachable_code) = &mut unreachable_code {
                unreachable_code.extend(expression.span());
            } else {
                r#type = expr_type;
            }
        }

        if let Some(unreachable_code) = unreachable_code {
            if unreachable_code.there_is_some {
                type_inference.report(self.metadata.unreachable_code(unreachable_code));
            }
        }

        r#type
    }
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

/// Unreachable code analysis data
pub struct UnreachableCode {
    /// True if there is actually some unreachable code
    pub there_is_some: bool,
    /// The span of the unreachable code
    pub unreachable_code: Option<Span>,
    /// The span of the reason code being unreachable
    pub reason: Option<Span>,
}

impl UnreachableCode {
    fn extend(&mut self, span: Option<&Span>) {
        self.there_is_some = true;
        if let Some(span) = span {
            if let Some(unreachable_span) = &mut self.unreachable_code {
                if unreachable_span.0 == span.0 {
                    unreachable_span.extend(span);
                }
            } else {
                self.unreachable_code = Some(span.clone());
            }
        }
    }
}

declare_metadata! {
    /// Frontend metadata for block expression
    trait BlockMetadata {
        /// Callback of unreachable code warning
        fn unreachable_code(&self, unreachable_code: UnreachableCode) -> Report {
            Report::build(ReportKind::Warning)
                .with_code("potential_bugs::unreachable_code")
                .with_message("Unreachable code")
                .opt_label(unreachable_code.unreachable_code, |label| label.with_message("This code is unreachable").with_color(colors::Label))
                .opt_label(unreachable_code.reason, |label| label.with_message("Note: Unreachable beacuse of this").with_color(colors::Hint))
                .finish()
        }
    }
}
