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
    pub span: Span,
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
        span: Span,
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
            |else_branch| self.then_branch.get_type() | else_branch.get_type(),
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
            self.metadata.if_condition_not_bool(
                type_inference,
                IfConditionNotBool {
                    condition_type,

                    src: self.condition.span().named_source(),
                    condition_span: self.condition.span().source_span(),
                },
            );
        }
        let then_type = self.then_branch.finish_and_check_types(type_inference);
        if let Some(else_branch) = &mut self.else_branch {
            let else_type = else_branch.finish_and_check_types(type_inference);
            if !else_type.morphs(&then_type) {
                self.metadata.else_branch_type_mismatch(
                    type_inference,
                    ElseBranchTypeMismatch {
                        then_type: then_type.clone(),
                        else_type,

                        src: else_branch.span().named_source(),
                        else_span: else_branch.span().source_span(),
                        then_span: self.then_branch.span().source_span(),
                    },
                );
            }
            then_type
        } else {
            Type::unit()
        }
    }
}

#[derive(Error, Debug, Diagnostic)]
#[error("If condition should be of type 'bool', but it is of type '{condition_type}'")]
#[diagnostic(code(typechecking::if_else::condition_not_bool))]
/// If condition is not bool
pub struct IfConditionNotBool {
    /// Type of the condition
    pub condition_type: Type,

    #[source_code]
    /// File where the error occurred
    pub src: NamedSource<Src>,
    #[label("Here")]
    /// Span of the condition
    pub condition_span: SourceSpan,
}

#[derive(Error, Debug, Diagnostic)]
#[error("Else branch type mismatch: Expected '{then_type}', got '{else_type}'")]
#[diagnostic(code(typechecking::if_else::else_branch_type_mismatch))]
/// Else branch type mismatch
pub struct ElseBranchTypeMismatch {
    /// Type of then branch
    pub then_type: Type,
    /// Type of else branch
    pub else_type: Type,

    #[source_code]
    /// File where the error occurred
    pub src: NamedSource<Src>,
    #[label("Here")]
    /// Span of else branch
    pub else_span: SourceSpan,
    #[label("Expected because of this")]
    /// Span of then branch
    pub then_span: SourceSpan,
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
        Diagnostics:
        /// Callback of if condition not bool error
        if_condition_not_bool(IfConditionNotBool) abort_compilation;
        /// Callback of else branch type mismatch
        else_branch_type_mismatch(ElseBranchTypeMismatch) abort_compilation;
    }
}
