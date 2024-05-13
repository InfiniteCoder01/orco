use super::*;
use ir::expression::Variable;
use std::sync::Arc;

/// A reference to a function
pub type FunctionReference = Arc<Spanned<ir::symbol::Function>>;
/// A reference to an external function
pub type ExternFunctionReference = Arc<Spanned<ir::symbol::function::Signature>>;

/// Symbol reference
#[derive(Clone, Debug)]
pub enum SymbolReference {
    /// Symbol, that hasn't been declared yet
    /// Use it to reference a symbol when generating IR
    Undeclared(Span),
    /// Variable
    Variable(Variable),
    /// Function
    Function(FunctionReference),
    /// External function
    ExternFunction(ExternFunctionReference),
}

impl SymbolReference {
    /// Get the type that the symbol evaluates to when accessed
    pub fn get_type(&self) -> ir::Type {
        match self {
            SymbolReference::Undeclared(_) => ir::Type::Error,
            SymbolReference::Variable(variable) => variable.r#type.lock().unwrap().clone(),
            SymbolReference::Function(function) => function.signature.get_type(),
            SymbolReference::ExternFunction(signature) => signature.get_type(),
        }
    }

    /// Infer types for this symbol
    pub fn infer_types(&mut self, type_inference: &mut TypeInference) -> ir::Type {
        match self {
            SymbolReference::Undeclared(name) => {
                if let Some(symbol) = type_inference.get_symbol(name) {
                    *self = symbol;
                    self.infer_types(type_inference)
                } else {
                    ir::Type::Error
                }
            }
            SymbolReference::Variable(variable) => variable.r#type.lock().unwrap().clone(),
            SymbolReference::Function(function) => function.signature.get_type(),
            SymbolReference::ExternFunction(signature) => signature.get_type(),
        }
    }

    /// Finish and check types
    pub fn finish_and_check_types(
        &self,
        span: Span,
        type_inference: &mut TypeInference,
    ) -> ir::Type {
        match self {
            SymbolReference::Undeclared(name) => {
                type_inference.reporter.report_type_error(
                    format!("Symbol '{}' was not declared in this scope", name),
                    span,
                    vec![],
                );
                ir::Type::Error
            }
            SymbolReference::Variable(variable) => variable.r#type.lock().unwrap().clone(),
            SymbolReference::Function(function) => function.signature.get_type(),
            SymbolReference::ExternFunction(function) => function.get_type(),
        }
    }
}

impl std::fmt::Display for SymbolReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Undeclared(span) => write!(f, "<undeclared symbol '{}'>", span),
            Self::Variable(variable) => {
                let show_id =
                    std::env::var("ORCO_SHOW_VAR_ID").map_or(false, |show_id| show_id == "1");
                if show_id {
                    write!(f, "{} (#{})", variable.name, variable.id.lock().unwrap())
                } else {
                    write!(f, "{}", variable.name)
                }
            }
            Self::Function(function) => write!(
                f,
                "{}",
                function
                    .signature
                    .name
                    .as_deref()
                    .unwrap_or("<anonymous function>")
            ),
            Self::ExternFunction(function) => write!(
                f,
                "{}",
                function.name.as_deref().unwrap_or("<anonymous function>")
            ),
        }
    }
}
