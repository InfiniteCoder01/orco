use super::*;

/// If expression (and ternary operator)
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct IfExpression {
    /// Condition
    pub condition: Box<Expression>,
    /// Then branch
    pub then_branch: Box<Expression>,
    /// Else branch
    pub else_branch: Option<Box<Expression>>,
    /// Span of the expression
    #[derivative(Debug = "ignore")]
    pub span: Option<Span>,
    /// Metadata
    #[derivative(Debug = "ignore")]
    pub metadata: Box<dyn IfMetadata>,
}

impl IfExpression {
    /// Create a new if expression
    pub fn new(
        condition: Box<Expression>,
        then_branch: Box<Expression>,
        else_branch: Option<Box<Expression>>,
        span: Option<Span>,
        metadata: impl IfMetadata + 'static,
    ) -> Self {
        Self {
            condition,
            then_branch,
            else_branch,
            span,
            metadata: Box::new(metadata),
        }
    }

    /// Get the type this exprssion (ternary) evaluates to
    pub fn get_type(&self) -> Type {
        self.else_branch.as_ref().map_or_else(
            || Type::Unit,
            |else_branch| self.then_branch.get_type() | &else_branch.get_type(),
        )
    }

    /// Infer the type for this expression
    pub fn infer_types(&mut self, type_inference: &mut TypeInference) -> Type {
        let condition_type = self.condition.infer_types(type_inference);
        type_inference.equate(&condition_type, &Type::Bool);
        // type_inference.push_scope();
        let then_type = self.then_branch.infer_types(type_inference);
        // type_inference.pop_scope();
        if let Some(else_branch) = &mut self.else_branch {
            // type_inference.push_scope();
            let else_type = else_branch.infer_types(type_inference);
            // type_inference.pop_scope();
            type_inference.equate(&then_type, &else_type)
        } else {
            type_inference.equate(&then_type, &Type::Unit);
            Type::unit()
        }
    }

    /// Finish and check types
    pub fn finish_and_check_types(&mut self, type_inference: &mut TypeInference) -> Type {
        let condition_type = self.condition.finish_and_check_types(type_inference);
        if !condition_type.morphs(&Type::Bool) {
            type_inference.report(
                self.metadata
                    .if_condition_not_bool(&condition_type, self.condition.span().cloned()),
            );
        }
        let then_type = self.then_branch.finish_and_check_types(type_inference);
        if let Some(else_branch) = &mut self.else_branch {
            let else_type = else_branch.finish_and_check_types(type_inference);
            if !else_type.morphs(&then_type) {
                type_inference.report(self.metadata.else_branch_type_mismatch(
                    &then_type,
                    &else_type,
                    self.then_branch.span().cloned(),
                    else_branch.span().cloned(),
                ));
            }
            then_type
        } else {
            Type::unit()
        }
    }
}

impl std::fmt::Display for IfExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "if {} {}", self.condition, self.then_branch)?;
        if let Some(else_branch) = &self.else_branch {
            write!(f, " else {}", else_branch)?;
        }
        Ok(())
    }
}

declare_metadata! {
    /// Frontend metadata for if expression
    trait IfMetadata {
        /// Callback of if condition not bool error
        fn if_condition_not_bool(&self, condition_type: &Type, span: Option<Span>) -> Report {
            Report::build(ReportKind::Error)
                .with_code("typechecking::if_condition_not_bool")
                .with_message(format!("If condition should be of type 'bool', but it is of type '{condition_type}'"))
                .opt_label(span, |label| label.with_message("Here").with_color(colors::Label))
                .finish()
        }

        /// Callback of else branch type mismatch
        fn else_branch_type_mismatch(&self, then_type: &Type, else_type: &Type, then_span: Option<Span>, else_span: Option<Span>) -> Report {
            Report::build(ReportKind::Error)
                .with_code("typechecking::if_else_branch_type_mismatch")
                .with_message(format!("If-Else branch type mismatch: Expected '{then_type}', got '{else_type}'"))
                .opt_label(else_span, |label| label.with_message("Here").with_color(colors::Got))
                .opt_label(then_span, |label| label.with_message("Expected because of this").with_color(colors::Expected))
                .finish()
        }
    }
}
