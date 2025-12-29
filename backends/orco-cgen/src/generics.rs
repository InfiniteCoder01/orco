use crate::{BackendContext, SymbolKind};

// FIXME: Proof of concept
/// [`crate::Backend`] wrapper that wraps all symbols in generic macros.
/// For more info, see [`orco::Backend::generic`]
pub(super) struct Wrapper<'a> {
    /// Original backend
    pub backend: &'a crate::Backend,
    /// Macro params
    pub params: Vec<orco::Symbol>,
}

impl BackendContext for Wrapper<'_> {
    fn backend(&self) -> &crate::Backend {
        self.backend
    }

    /// Adds a symbol, wrapping it with generics
    fn symbol(&self, name: orco::Symbol, kind: SymbolKind) {
        self.backend.symbol(
            name,
            SymbolKind::Generic {
                params: self.params.clone(),
                symbol: Box::new(kind),
            },
        );
    }

    fn escape(&self, symbol: orco::Symbol) -> String {
        crate::escape(&symbol.replace('#', "##_##"))
    }

    fn intern_type(&self, ty: &mut orco::Type, named: bool, replace_unit: bool) {
        // TODO: Interned types in generics
        self.backend.intern_type(ty, named, replace_unit)
    }
}

impl orco::Backend for Wrapper<'_> {
    fn function(
        &self,
        name: orco::Symbol,
        mut params: Vec<(Option<orco::Symbol>, orco::Type)>,
        mut return_type: orco::Type,
    ) -> impl orco::codegen::BodyCodegen {
        for (_, ty) in &mut params {
            self.backend.intern_type(ty, false, false);
        }
        self.backend.intern_type(&mut return_type, false, true);

        crate::codegen::Codegen::new(
            self,
            name,
            crate::symbols::FunctionSignature {
                params,
                return_type,
            },
        )
    }

    fn type_(&self, name: orco::Symbol, mut ty: orco::Type) {
        self.backend.intern_type(&mut ty, true, false);
        self.symbol(name, SymbolKind::Type(ty));
    }

    fn generic(&self, params: Vec<orco::Symbol>) -> impl orco::Backend {
        Self {
            backend: self.backend,
            params: self
                .params
                .iter()
                .cloned()
                .chain(params.into_iter())
                .collect(),
        }
    }
}
