use crate::{Symbol, Type};

/// Id of a symbol (index into list of referenced symbols).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolId(pub u32);

impl std::fmt::Display for SymbolId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}

/// Reference to a symbol.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolUse {
    /// Symbol name.
    pub name: Symbol,
    /// A set of generic arguments.
    pub generics: Vec<Type>,
    /// Cached symbol type (instantiated)
    pub ty: Type,
}

impl std::fmt::Display for SymbolUse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            self.name,
            crate::types::fmt_generic_args(&self.generics)
        )
    }
}

impl super::Body {
    /// Retrieve the [`SymbolUse`] by ID.
    pub fn symbol(&self, id: SymbolId) -> &SymbolUse {
        &self
            .symbols
            .get(id.0 as usize)
            .unwrap_or_else(|| panic!("invalid symbol id {id}"))
    }

    /// Reference a symbol from the global namespace, adding it to the list of symbols
    /// (unless already there), returns the ID to be used with [`Self::symbol`].
    pub fn use_symbol(
        &mut self,
        name: Symbol,
        generics: Vec<Type>,
        module: &crate::Module,
    ) -> SymbolId {
        let symbol = (name, generics);
        if let Some(id) = self.interned_symbols.get(&symbol) {
            return *id;
        }

        let id = SymbolId(self.symbols.len() as _);
        self.interned_symbols.insert(symbol.clone(), id);
        let (name, generics) = symbol;
        self.symbols.push(SymbolUse {
            name,
            generics,
            ty: Type::Error,
        });
        self.refresh_symbol_type(id, module);
        id
    }

    /// Recomputes type of the symbol use, to be up to date with the global.
    pub fn refresh_symbol_type(&mut self, id: SymbolId, module: &crate::Module) {
        let symbol = self
            .symbols
            .get_mut(id.0 as usize)
            .unwrap_or_else(|| panic!("invalid symbol id {id}"));

        let functions = module.functions.pin();
        let func = functions
            .get(&symbol.name)
            .unwrap_or_else(|| panic!("undefined symbol {}", symbol.name));
        symbol.ty = func
            .ptr_type()
            .copy_instantiate(&func.generic_map(&symbol.generics));
    }
}
