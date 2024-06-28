use super::*;
use diagnostics::*;
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
    Undeclared(Path),
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
    pub fn infer_types(
        &mut self,
        type_inference: &mut TypeInference,
        metadata: &mut dyn SymbolMetadata,
    ) -> ir::Type {
        match self {
            SymbolReference::Undeclared(name) => {
                if let Some(symbol) = type_inference.resolve_symbol(name) {
                    *self = symbol;
                    self.infer_types(type_inference, metadata)
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
        metadata: &mut dyn SymbolMetadata,
    ) -> ir::Type {
        match self {
            SymbolReference::Undeclared(path) => {
                metadata.symbol_not_found(
                    type_inference,
                    SymbolNotFound {
                        path: path.clone(),
                        src: span.named_source(),
                        span: span.source_span(),
                    },
                );
                ir::Type::Error
            }
            SymbolReference::Variable(variable) => variable.r#type.lock().unwrap().clone(),
            SymbolReference::Function(function) => function.signature.get_type(),
            SymbolReference::ExternFunction(function) => function.get_type(),
        }
    }
}

#[derive(Error, Debug, Diagnostic)]
#[error("Symbol '{path}' was not declared in this scope'")]
#[diagnostic(code(symbol::symbol_not_found))]
/// Symbol not found
pub struct SymbolNotFound {
    /// Path of the symbol
    pub path: Path,

    #[source_code]
    /// File where the error occurred
    pub src: NamedSource<Src>,
    #[label("Here")]
    /// Span of the symbol
    pub span: SourceSpan,
}

impl std::fmt::Display for SymbolReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Undeclared(path) => write!(f, "<undeclared symbol '{}'>", path),
            Self::Variable(variable) => {
                let show_id =
                    std::env::var("ORCO_SHOW_VAR_ID").map_or(false, |show_id| show_id == "1");
                if show_id {
                    write!(f, "{} (#{})", variable.name, variable.id.lock().unwrap())
                } else {
                    write!(f, "{}", variable.name)
                }
            }
            Self::Function(function) => write!(f, "{}", function.signature.name),
            Self::ExternFunction(function) => write!(f, "{}", function.name),
        }
    }
}

use downcast_rs::{impl_downcast, Downcast};
use dyn_clone::{clone_trait_object, DynClone};
declare_metadata! {
    /// Frontend metadata for symbols
    trait SymbolMetadata {
        Errors:
        /// Callback of symbol not found error
        symbol_not_found(SymbolNotFound)
    }
}
