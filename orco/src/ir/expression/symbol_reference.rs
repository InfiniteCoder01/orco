use super::*;

/// Symbol reference
#[derive(Clone, Debug)]
pub enum SymbolReference {
    /// Unresolved symbol, should be resolved during type inference pass
    Unresolved(Path),
    /// Reference to a symbol
    Symbol(InternalPointer<std::sync::Mutex<Symbol>>),
}

impl SymbolReference {
    /// Get the type that the symbol evaluates to when accessed
    pub fn get_type(&self) -> ir::Type {
        match self {
            SymbolReference::Unresolved(_) => ir::Type::Error,
            SymbolReference::Symbol(symbol) => symbol.lock().unwrap().value.get_type(),
        }
    }

    /// Infer types for this symbol
    pub fn infer_types(
        &mut self,
        type_inference: &mut TypeInference,
        metadata: &mut dyn SymbolMetadata,
    ) -> ir::Type {
        match self {
            SymbolReference::Unresolved(path) => {
                if let Some(symbol) = metadata.resolve_symbol(type_inference, path) {
                    *self = symbol;
                    self.infer_types(type_inference, metadata)
                } else {
                    ir::Type::Error
                }
            }
            _ => self.get_type(),
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
            SymbolReference::Unresolved(path) => {
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
            _ => self.get_type(),
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
            Self::Unresolved(path) => write!(f, "<unresolved symbol '{}'>", path),
            Self::Symbol(symbol) => write!(f, "<symbol {}>", symbol.lock().unwrap().name),
            // Self::Variable(variable) => {
            //     let show_id =
            //         std::env::var("ORCO_SHOW_VAR_ID").map_or(false, |show_id| show_id == "1");
            //     if show_id {
            //         write!(f, "{} (#{})", variable.name, variable.id.lock().unwrap())
            //     } else {
            //         write!(f, "{}", variable.name)
            //     }
            // }
        }
    }
}

declare_metadata! {
    /// Frontend metadata for symbols
    trait SymbolMetadata {
        /// Symbol resolver (resolves variables, functions, etc.)
        fn resolve_symbol(&self, type_inference: &mut TypeInference, path: &Path) -> Option<SymbolReference> {
            let start = path.0.first().expect("Trying to resolve an empty path!");
            // if let Some(symbol) = type_inference.get_symbol(start) {
            //     return Some(symbol);
            // }
            if let Some(symbol) = type_inference.current_module.symbols.get(start) {
                return Some(SymbolReference::Symbol(InternalPointer::new(symbol)));
            }
            None
        }

        Diagnostics:
        /// Callback of symbol not found error
        symbol_not_found(SymbolNotFound)
    }
}

/// Pointer to interanl IR data, use with care!
#[derive(Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub struct InternalPointer<T>(*const T);

impl<T> InternalPointer<T> {
    /// Create a new internal pointer. Only use this for
    /// IR nodes that have to be referenced anywhere in IR
    pub fn new(value: &Box<T>) -> Self {
        Self(value.as_ref() as _)
    }
}

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
