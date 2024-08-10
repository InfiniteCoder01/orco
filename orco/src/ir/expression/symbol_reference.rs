use super::*;

/// Symbol reference
#[derive(Clone, Debug)]
pub enum SymbolReference {
    /// Unresolved symbol, should be resolved during type inference pass
    Unresolved(Name),
    /// Unresolved symbol (from a scope), should be resolved during type inference pass
    ScopeAccess(Box<Spanned<SymbolReference>>, Name),
    /// Reference to a symbol
    Symbol(InternalPointer<std::sync::RwLock<Symbol>>),
    /// Reference to a symbol
    Variable(InternalPointer<VariableDeclaration>),
}

impl SymbolReference {
    /// Get the type that the symbol evaluates to when accessed
    pub fn get_type(&self) -> ir::Type {
        match self {
            SymbolReference::Unresolved(..) => ir::Type::Error,
            SymbolReference::ScopeAccess(..) => ir::Type::Error,
            SymbolReference::Symbol(symbol) => {
                if symbol::check_for_recursion(symbol) {
                    return Type::Error;
                }

                let symbol = symbol.try_read().unwrap();
                let value = symbol.evaluated.as_ref();
                match (&symbol.r#type.inner, value) {
                    (Type::Function, Some(value)) => {
                        value.as_ref::<Function>().signature.get_type()
                    }
                    (Type::ExternFunction, Some(value)) => {
                        value.as_ref::<ExternFunction>().signature.get_type()
                    }
                    (Type::Function | Type::ExternFunction, None) => Type::Error,
                    (r#type, _) => r#type.clone(),
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
            SymbolReference::Unresolved(name) => {
                if let Some(symbol) = metadata.resolve_global_symbol(type_inference, name) {
                    *self = symbol;
                    self.infer_types(type_inference, metadata)
                } else {
                    ir::Type::Error
                }
            }
            SymbolReference::ScopeAccess(scope, name) => {
                scope.inner.infer_types(type_inference, metadata);
                if let Some(symbol) = metadata.resolve_scoped_symbol(type_inference, scope, name) {
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
                type_inference.complete(&mut r#type);
                r#type.clone()
            }
        }
    }

    /// Finish and check types
    pub fn finish_and_check_types(
        &self,
        span: &Option<Span>,
        type_inference: &mut TypeInference,
        metadata: &mut dyn SymbolMetadata,
    ) -> ir::Type {
        match self {
            SymbolReference::Unresolved(name) => {
                type_inference.report(metadata.global_symbol_not_found(name, span.clone()));
                ir::Type::Error
            }
            SymbolReference::ScopeAccess(scope, name) => {
                scope
                    .inner
                    .finish_and_check_types(&scope.span, type_inference, metadata);
                type_inference.report(metadata.scoped_symbol_not_found(scope, name, span.clone()));
                ir::Type::Error
            }
            SymbolReference::Symbol(symbol) => {
                if symbol::check_for_recursion(symbol) {
                    type_inference.report(metadata.recursive_evaluation(
                        span.as_ref().unwrap_or(&Span::new("Unknow")),
                        span.clone(),
                    ));
                    Type::Error
                } else {
                    self.get_type()
                }
            }
            _ => self.get_type(),
        }
    }
}

impl std::fmt::Display for SymbolReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unresolved(name) => write!(f, "<{name}>"),
            Self::ScopeAccess(scope, name) => write!(f, "{scope}::{name}"),
            Self::Symbol(symbol) => write!(f, "{}", symbol.try_read().unwrap().path),
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
        /// Resolve symbol
        fn resolve_global_symbol(&self, type_inference: &mut TypeInference, name: &Name) -> Option<SymbolReference> {
            if let Some(symbol) = type_inference.get_symbol(name) {
                return Some(symbol);
            }
            if let Some(symbol) = type_inference.current_module.symbols.get(name) {
                return Some(SymbolReference::Symbol(InternalPointer::new(symbol.as_ref())));
            }
            None
        }

        /// Resolve symbol inside of a scope
        fn resolve_scoped_symbol(&self, type_inference: &mut TypeInference, scope: &Spanned<SymbolReference>, name: &Name) -> Option<SymbolReference> {
                let _ = type_inference;
                match scope.inner {
                    SymbolReference::Symbol(symbol) => {
                        let symbol = symbol.try_read().unwrap();
                        let Some(value) = &symbol.evaluated else { return None; };
                        match symbol.r#type.inner {
                            Type::Module => {
                                let module = value.as_ref::<Module>();
                                module.symbols.get(name).map(|symbol| SymbolReference::Symbol(InternalPointer::new(symbol.as_ref())))
                            }
                            _ => None,
                        }
                    }
                    _ => None,
                }
            }

        /// Callback of global symbol not found error
        fn global_symbol_not_found(&self, name: &Name, span: Option<Span>) -> Report {
            Report::build(ReportKind::Error)
                .with_code("symbol::symbol_not_found")
                .with_message(format!("Symbol '{name}' was not declared in this scope"))
                .opt_label(span, |label| label.with_message("Here").with_color(colors::Label))
                .finish()
        }

        /// Callback of global symbol not found error
        fn scoped_symbol_not_found(&self, scope: &Spanned<SymbolReference>, name: &Name, span: Option<Span>) -> Report {
            Report::build(ReportKind::Error)
                .with_code("symbol::symbol_not_found")
                .with_message(format!("Symbol '{name}' could not be found in '{scope}'"))
                .opt_label(span, |label| label.with_message("Here").with_color(colors::Label))
                .finish()
        }

        /// Callback of recursive evaluation error
        fn recursive_evaluation(&self, name: &Name, span: Option<Span>)  -> Report {
            Report::build(ReportKind::Error)
                .with_code("symbol::recursive_evaluation")
                .with_message(format!("Recursive use of a constexpr symbol '{name}' in it's evaluation"))
                .opt_label(span, |label| label.with_message("Here").with_color(colors::Label))
                .finish()
        }
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
