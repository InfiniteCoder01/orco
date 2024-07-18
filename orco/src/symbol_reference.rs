use super::*;
use derivative::Derivative;
use diagnostics::*;

/// Pointer to interanl IR data, use with care!
#[derive(Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub struct InternalPointer<T>(pub(crate) *const T);
unsafe impl<T: Send> Send for InternalPointer<T> {}
unsafe impl<T: Sync> Sync for InternalPointer<T> {}
impl<T: std::fmt::Debug> std::fmt::Debug for InternalPointer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> std::ops::Deref for InternalPointer<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

/// A reference to a variable
pub type VariableReference = InternalPointer<ir::expression::VariableDeclaration>;
/// A reference to a function
pub type FunctionReference = InternalPointer<ir::symbol::Function>;
/// A reference to an external function
pub type ExternFunctionReference = InternalPointer<ir::symbol::function::Signature>;

/// Symbol reference
#[derive(Clone, Debug)]
pub enum SymbolReference {
    /// Symbol, that hasn't been declared yet
    /// Use it to reference a symbol when generating IR
    Undeclared(Path),
    /// Variable
    Variable(VariableReference),
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
                if let Some(symbol) = metadata.resolve_symbol(type_inference, name) {
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
#[error("Symbol '{path}' was not declared in this scope")]
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
        /// Symbol resolver (resolves variables, functions, etc.)
        fn resolve_symbol(&self, type_inference: &mut TypeInference, path: &Path) -> Option<SymbolReference> {
            let start = path.0.first().expect("Trying to resolve an empty path!");
            if let Some(symbol) = type_inference.get_symbol(start) {
                return Some(symbol);
            }
            if let Some(symbol) = type_inference.current_module.symbol_map.get(start) {
                return Some(symbol.first().unwrap().clone());
            }
            None
        }

        Diagnostics:
        /// Callback of symbol not found error
        symbol_not_found(SymbolNotFound)
    }
}
