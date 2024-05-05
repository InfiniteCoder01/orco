use super::*;

/// Variable declaration statement
#[derive(Clone, Debug)]
pub struct VariableDeclaration {
    /// Variable name
    pub name: Span,
    /// Variable ID, just a counting up number assigned automatically, when calling
    /// [`crate::variable_mapper::VariableMapper::declare_variable`]
    /// Useful for some backends
    pub id: VariableID,
    /// Is variable mutable?
    pub mutable: Spanned<bool>,
    /// Variable type
    pub r#type: Spanned<Type>,
    /// Initial value (optional (I wish it was nesessarry))
    pub value: Option<Box<Expression>>,
}

/// Variable reference, used in variable access
pub type VariableReference = Arc<Spanned<Mutex<VariableDeclaration>>>;
/// Variable ID, for more information see [`VariableDeclaration::id`]
pub type VariableID = u64;

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
                &format!("variable '{}'", self.name),
                self.name.clone(),
            );
            if !value_type.morphs(&self.r#type) {
                type_inference.reporter.report_type_error(
                    format!(
                        "Type mismatch in variable declaration: Expected '{}', got '{}'",
                        self.r#type.inner, value_type
                    ),
                    value.span(),
                    vec![("Expected because of this", self.r#type.span.clone())],
                );
            }
        } else {
            type_inference.finish(
                &mut self.r#type,
                &format!("variable '{}'", self.name),
                self.name.clone(),
            );
        }
        Type::Unit
    }
}

impl std::fmt::Display for VariableDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let show_id = std::env::var("ORCO_SHOW_VAR_ID").map_or(false, |show_id| show_id == "1");
        write!(f, "let ")?;
        if self.mutable.inner {
            write!(f, "mut ")?;
        }
        write!(f, "{}", self.name)?;
        if show_id {
            write!(f, " (#{})", self.id)?;
        }
        write!(f, ": {}", self.r#type.inner)?;
        if let Some(value) = &self.value {
            write!(f, " = {}", value)?;
        }
        Ok(())
    }
}

/// VaraibleReference extensions
pub trait VariableReferenceExt {
    /// Get the type of this variable, not holding the lock
    fn r#type(&self) -> Type;
}

impl VariableReferenceExt for VariableReference {
    fn r#type(&self) -> Type {
        self.lock().unwrap().r#type.inner.clone()
    }
}
