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
pub struct SymbolRef {
    /// Symbol name.
    pub name: Symbol,
    /// A set of generic arguments.
    pub generics: Vec<Type>,
}

impl std::fmt::Display for SymbolRef {
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
    /// Reference a symbol from the global namespace, adding it to the list of symbols
    /// (unless already there), returns the ID to be used with [`Self::symbol`].
    pub fn use_symbol(&mut self, name: Symbol, generics: Vec<Type>) -> SymbolId {
        let symbol = SymbolRef { name, generics };
        if let Some(id) = self.interned_symbols.get(&symbol) {
            return *id;
        }

        let id = SymbolId(self.symbols.len() as _);
        self.symbols.push(symbol.clone());
        self.interned_symbols.insert(symbol, id);
        id
    }

    /// Retrieve the [`SymbolRef`] by ID.
    pub fn symbol(&self, id: SymbolId) -> &SymbolRef {
        &self
            .symbols
            .get(id.0 as usize)
            .unwrap_or_else(|| panic!("invalid symbol id {id}"))
    }
}
