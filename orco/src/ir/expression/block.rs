use super::*;

/// Block expression, contains multiple expressions (something along { expr1; expr2; })
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Block {
    /// Block content
    pub expressions: Vec<Expression>,
}

impl Block {
    /// Create a new block
    pub fn new(expressions: Vec<Expression>) -> Self {
        Self { expressions }
    }

    /// Get the type this block evaluates to
    pub fn get_type(&self, root: &crate::ir::Module) -> Type {
        for expression in &self.expressions {
            if expression.get_type(root) == Type::Never {
                return Type::Never;
            }
        }
        Type::Unit
    }

    /// Infer types
    pub fn infer_types(
        &mut self,
        _target_type: &Type,
        type_inference: &mut TypeInference,
    ) -> Type {
        let mut r#type = Type::Unit;
        for expression in &mut self.expressions {
            let expr_type = expression.infer_types(&Type::Wildcard, type_inference);
            if expr_type == Type::Never {
                r#type = Type::Never;
            }
        }

        r#type
    }

    /// Finish and check types
    pub fn finish_and_check_types(
        &mut self,
        type_inference: &mut TypeInference,
    ) -> Type {
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

        if let Some(span) = unreachable_span {
            let mut colors = ColorGenerator::new();
            let report = Report::build(ReportKind::Warning, span.0.clone(), span.1.start)
                .with_message("This code is unreachable")
                .with_label(
                    Label::new(span)
                        .with_message("This")
                        .with_color(colors.next()),
                )
                .finish();
            type_inference.reporter.report(report);
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
        write!(f, "}}")?;
        Ok(())
    }
}
