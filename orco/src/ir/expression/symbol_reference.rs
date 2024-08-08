use super::*;

/// Symbol reference
#[derive(Clone, Debug)]
pub enum SymbolReference {
    /// Unresolved symbol, should be resolved during type inference pass
    Unresolved(Path),
    /// Reference to a symbol
    Symbol(InternalPointer<std::sync::RwLock<Symbol>>),
    /// Reference to a symbol
    Variable(InternalPointer<VariableDeclaration>),
}

impl SymbolReference {
    /// Get the type that the symbol evaluates to when accessed
    pub fn get_type(&self) -> ir::Type {
        match self {
            SymbolReference::Unresolved(_) => ir::Type::Error,
            SymbolReference::Symbol(symbol) => {
                if symbol::check_for_recursion(symbol) {
                    return Type::Error;
                }

                let symbol = symbol.try_read().unwrap();
                let r#type = symbol.value.get_type();
                let Some(value) = symbol.evaluated.as_ref() else {
                    return Type::Error;
                };
                match r#type {
                    Type::Function => value.as_ref::<Function>().signature.get_type(),
                    Type::ExternFunction => value.as_ref::<ExternFunction>().signature.get_type(),
                    r#type => r#type,
                }
            }
            SymbolReference::Variable(variable) => variable.r#type.try_lock().unwrap().clone(),
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
            SymbolReference::Symbol(symbol) => {
                if symbol::check_for_recursion(symbol) {
                    return Type::Error;
                }
                symbol::ensure_evaluated(symbol, type_inference);
                self.get_type()
            }
            SymbolReference::Variable(variable) => {
                let mut r#type = variable.r#type.inner.try_lock().unwrap();
                *r#type = type_inference.complete(r#type.clone());
                r#type.clone()
            }
        }
    }

    /// Finish and check types
    pub fn finish_and_check_types(
        &self,
        span: Option<Span>,
        type_inference: &mut TypeInference,
        metadata: &mut dyn SymbolMetadata,
    ) -> ir::Type {
        match self {
            SymbolReference::Unresolved(path) => {
                metadata.symbol_not_found(
                    type_inference,
                    SymbolNotFound {
                        path: path.clone(),
                        src: span.as_ref().unwrap().named_source(),
                        span: span.as_ref().unwrap().source_span(),
                    },
                );
                ir::Type::Error
            }
            SymbolReference::Symbol(symbol) => {
                if symbol::check_for_recursion(symbol) {
                    metadata.recursive_evaluation(
                        type_inference,
                        RecursiveEvaluation {
                            name: span.as_ref().unwrap().clone(),
                            src: span.as_ref().unwrap().named_source(),
                            span: span.as_ref().unwrap().source_span(),
                        },
                    );
                    Type::Error
                } else {
                    self.get_type()
                }
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

#[derive(Error, Debug, Diagnostic)]
#[error("Recursive use of a constexpr symbol '{name}' in it's evaluation")]
#[diagnostic(code(symbol::recursive_evaluation))]
/// Recursive use of a constexpr symbol in it's evaluation
pub struct RecursiveEvaluation {
    /// Name of the symbol
    pub name: Span,

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
            Self::Symbol(symbol) => write!(f, "<symbol {}>", symbol.try_read().unwrap().name),
            Self::Variable(variable) => {
                let show_id =
                    std::env::var("ORCO_SHOW_VAR_ID").map_or(false, |show_id| show_id == "1");
                if show_id {
                    write!(
                        f,
                        "{} (#{})",
                        variable.name,
                        variable.id.try_lock().unwrap()
                    )
                } else {
                    write!(f, "{}", variable.name)
                }
            }
        }
    }
}

declare_metadata! {
    /// Frontend metadata for symbols
    trait SymbolMetadata {
        /// Symbol resolver (resolves variables, functions, etc.)
        fn resolve_symbol(&self, type_inference: &mut TypeInference, path: &Path) -> Option<SymbolReference> {
            let start = path.0.first().expect("Trying to resolve an empty path!");
            if let Some(symbol) = type_inference.get_symbol(start) {
                return Some(symbol);
            }
            if let Some(symbol) = type_inference.current_module.symbols.get(start) {
                return Some(SymbolReference::Symbol(InternalPointer::new(symbol.as_ref())));
            }
            None
        }

        Diagnostics:
        /// Callback of symbol not found error
        symbol_not_found(SymbolNotFound) abort_compilation;
        /// Callback of recursive evaluation error
        recursive_evaluation(RecursiveEvaluation) abort_compilation;
    }
}

/// Pointer to interanl IR data, use with care!
#[derive(Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub struct InternalPointer<T>(*const T);

impl<T> InternalPointer<T> {
    /// Create a new internal pointer. Only use this for
    /// IR nodes that have to be referenced anywhere in IR
    #[allow(clippy::borrowed_box)]
    pub fn new(value: std::pin::Pin<&T>) -> Self {
        Self(&*value as _)
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
