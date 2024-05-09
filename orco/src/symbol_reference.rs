use super::*;
use ir::Type;
use std::sync::{Arc, Mutex};

/// A symbol reference (function, variable, module, etc.)
#[derive(Clone, Debug)]
pub enum SymbolReference {
    /// Variable
    Variable(VariableReference),
    /// Function
    ExternFunction(ExternFunctionReference),
}

impl SymbolReference {
    /// Get the type this symbol evaluates to
    pub fn get_type(&self) -> Type {
        match self {
            Self::Variable(variable) => variable.lock().unwrap().r#type.inner.clone(),
            Self::ExternFunction(function) => function.get_type(),
        }
    }

    /// Infer types for this symbol reference (and deduce what it refers to)
    pub fn infer_types(
        &mut self,
        _target_type: &Type,
        _type_inference: &mut TypeInference,
    ) -> Type {
        match self {
            Self::Variable(variable) => variable.lock().unwrap().r#type.inner.clone(),
            Self::ExternFunction(function) => function.get_type(),
        }
    }

    /// Finish and check types for this symbol reference
    pub fn finish_and_check_types(&self, _type_inference: &mut TypeInference) -> Type {
        match self {
            Self::Variable(variable) => variable.lock().unwrap().r#type.inner.clone(),
            Self::ExternFunction(function) => function.get_type(),
        }
    }

    /// Get the span of this symbol
    pub fn span(&self) -> Span {
        match self {
            Self::Variable(variable) => variable.span.clone(),
            Self::ExternFunction(function) => function.span.clone(),
        }
    }
}

impl std::fmt::Display for SymbolReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Variable(variable) => {
                let variable = variable.lock().unwrap();
                let show_id =
                    std::env::var("ORCO_SHOW_VAR_ID").map_or(false, |show_id| show_id == "1");
                if show_id {
                    write!(f, "{} (#{})", variable.name, variable.id)
                } else {
                    variable.name.fmt(f)
                }
            }
            Self::ExternFunction(function) => function.span.fmt(f),
        }
    }
}

/// Function reference
pub type FunctionReference = Arc<Spanned<ir::symbol::Function>>;
/// Extern function reference
pub type ExternFunctionReference = Arc<Spanned<ir::symbol::function::Signature>>;
/// Variable reference
pub type VariableReference = Arc<Spanned<Mutex<ir::expression::VariableDeclaration>>>;

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
