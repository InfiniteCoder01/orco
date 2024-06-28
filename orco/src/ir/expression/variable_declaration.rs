use super::*;
use std::sync::Mutex;

/// Variable declaration statement
#[derive(Debug)]
pub struct VariableDeclaration {
    /// Variable name
    pub name: PathSegment,
    /// Variable ID, just a counting up number assigned automatically, when calling
    /// Useful for some backends
    pub id: Mutex<VariableId>,
    /// Is variable mutable?
    pub mutable: Spanned<bool>,
    /// Variable type
    pub r#type: Spanned<Mutex<Type>>,
    /// Initial value (optional (I wish it was nesessarry))
    pub value: Option<Mutex<Expression>>,
}

/// Variable ID, for more information see [`VariableDeclaration::id`]
pub type VariableId = u64;

impl VariableDeclaration {
    /// Create a new variable declaration
    pub fn new(
        name: Span,
        mutable: Spanned<bool>,
        r#type: Spanned<Type>,
        value: Option<Expression>,
    ) -> Self {
        Self {
            name,
            id: Mutex::new(0),
            mutable,
            r#type: r#type.map(Mutex::new),
            value: value.map(Mutex::new),
        }
    }

    /// Infer types
    pub fn infer_types(&self, type_inference: &mut TypeInference) -> Type {
        *self.id.lock().unwrap() = type_inference.new_variable_id();
        let mut r#type = self.r#type.inner.lock().unwrap();
        *r#type = type_inference.complete(r#type.clone());
        if let Some(value) = &self.value {
            let value_type = value.lock().unwrap().infer_types(type_inference);
            type_inference.equate(&r#type, &value_type);
        }
        Type::Unit
    }

    /// Finish and check types
    pub fn finish_and_check_types(&self, type_inference: &mut TypeInference) -> Type {
        let mut r#type = self.r#type.inner.lock().unwrap();
        type_inference.finish(
            &mut r#type,
            &format!("variable '{}'", self.name),
            self.name.clone(),
        );
        if let Some(value) = &self.value {
            let mut value = value.lock().unwrap();
            let value_type = value.finish_and_check_types(type_inference);
            if !value_type.morphs(&r#type) {
                type_inference.reporter.report_type_error(
                    format!(
                        "Type mismatch in variable declaration: Expected '{}', got '{}'",
                        r#type, value_type
                    ),
                    value.span(),
                    vec![("Expected because of this", self.r#type.span.clone())],
                );
            }
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
            write!(f, " (#{})", self.id.lock().unwrap())?;
        }
        write!(f, ": {}", self.r#type.lock().unwrap())?;
        if let Some(value) = &self.value {
            write!(f, " = {}", value.lock().unwrap())?;
        }
        Ok(())
    }
}

/// Variable (a reference to it's declaration)
pub type Variable = std::sync::Arc<Spanned<VariableDeclaration>>;
