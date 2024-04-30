use super::*;

/// Variable declaration statement
#[derive(Clone, Debug)]
pub struct VariableDeclaration {
    /// Variable name
    pub name: Spanned<String>,
    /// Is variable mutable?
    pub mutable: Spanned<bool>,
    /// Variable type
    pub r#type: Spanned<Type>,
    /// Initial value (optional (I wish it was nesessarry))
    pub value: Option<Box<Expression>>,
}

/// Variable reference, used in variable access
pub type VariableReference = Arc<Spanned<Mutex<VariableDeclaration>>>;

impl VariableDeclaration {
    /// Infer types
    pub fn infer_types(&mut self, type_inference: &mut TypeInference) -> Type {
        if let Some(value) = &mut self.value {
            let value_type = value.infer_types(&self.r#type, type_inference);
            self.r#type.inner = type_inference.complete(self.r#type.inner.clone());
            type_inference.equate(&self.r#type, &value_type);
        } else {
            self.r#type.inner = type_inference.complete(self.r#type.inner.clone());
        }
        Type::Unit
    }

    /// Finish and check types
    pub fn finish_and_check_types(&mut self, type_inference: &mut TypeInference) -> Type {
        if let Some(value) = &mut self.value {
            let value_type = value.finish_and_check_types(type_inference);
            type_inference.finish(
                &mut self.r#type,
                &format!("variable '{}'", self.name.inner),
                self.name.span.clone(),
            );
            if !value_type.morphs(&self.r#type) {
                type_inference.reporter.report_type_error(
                    format!(
                        "Type mismatch in variable declaration: Expected '{}', got '{}'",
                        self.r#type.inner, value_type
                    ),
                    value.span(),
                    Some(self.r#type.span.clone()),
                );
            }
        } else {
            type_inference.finish(
                &mut self.r#type,
                &format!("variable '{}'", self.name.inner),
                self.name.span.clone(),
            );
        }
        Type::Unit
    }
}

impl std::fmt::Display for VariableDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "let ")?;
        if self.mutable.inner {
            write!(f, "mut ")?;
        }
        write!(f, "{}: {}", self.name.inner, self.r#type.inner)?;
        if let Some(value) = &self.value {
            write!(f, " = {}", value)?;
        }
        Ok(())
    }
}
