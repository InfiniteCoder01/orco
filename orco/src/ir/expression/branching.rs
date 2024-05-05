use super::*;

/// If expression (and ternary operator)
#[derive(Clone, Debug)]
pub struct IfExpression {
    /// Condition
    pub condition: Box<Expression>,
    /// Then branch
    pub then_branch: Box<Expression>,
    /// Else branch
    pub else_branch: Option<Box<Expression>>,
}

impl IfExpression {
    /// Create a new if expression
    pub fn new(
        condition: Box<Expression>,
        then_branch: Box<Expression>,
        else_branch: Option<Box<Expression>>,
    ) -> Self {
        Self {
            condition,
            then_branch,
            else_branch,
        }
    }

    /// Get the type this exprssion (ternary) evaluates to
    pub fn get_type(&self, root: &crate::ir::Module) -> Type {
        self.else_branch.as_ref().map_or_else(
            || Type::Unit,
            |else_branch| self.then_branch.get_type(root) | else_branch.get_type(root),
        )
    }

    /// Infer the type for this expression
    pub fn infer_types(&mut self, target_type: &Type, type_inference: &mut TypeInference) -> Type {
        self.condition.infer_types(&Type::Bool, type_inference);
        let then_type = self.then_branch.infer_types(target_type, type_inference);
        if let Some(else_branch) = &mut self.else_branch {
            let else_type = else_branch.infer_types(target_type, type_inference);
            type_inference.equate(&then_type, &else_type)
        } else {
            Type::unit()
        }
    }

    /// Finish and check types
    pub fn finish_and_check_types(&mut self, type_inference: &mut TypeInference) -> Type {
        let condition_type = self.condition.finish_and_check_types(type_inference);
        if !condition_type.morphs(&Type::Bool) {
            type_inference.reporter.report_type_error(
                format!(
                    "If condition should be of type 'bool', but it is of type '{}'",
                    condition_type
                ),
                self.condition.span(),
                vec![],
            );
        }
        let then_type = self.then_branch.finish_and_check_types(type_inference);
        if let Some(else_branch) = &mut self.else_branch {
            let else_type = else_branch.finish_and_check_types(type_inference);
            if !else_type.morphs(&then_type) {
                type_inference.reporter.report_type_error(
                    format!(
                        "Else branch type mismatch: Expected '{}', got '{}'",
                        then_type, else_type
                    ),
                    else_branch.span().clone(),
                    vec![("Expected because of this", self.then_branch.span().clone())],
                );
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
