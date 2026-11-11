use std::collections::HashMap;

/// A single compilation unit.
/// Note: Be careful with mutating the types,
/// as [`Body`] caches them.
#[derive(Clone, Default, PartialEq)]
pub struct Module {
    #[allow(missing_docs)]
    pub types: HashMap<crate::ty::AliasId, crate::ty::Alias>,
    #[allow(missing_docs)]
    pub symbols: HashMap<SymbolId, Symbol>,
}

impl Module {
    #[allow(missing_docs)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Replaces the type alias by it's aliased type until can't anymore.
    /// Reveals the true identity of the type.
    pub fn inline_ty(&self, mut ty: crate::Type) -> crate::Type {
        while let crate::Type::Alias(id, generics) = ty {
            ty = self.types[&id].ty.copy_instantiate(&generics);
        }

        ty
    }
}

impl std::fmt::Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (idx, alias) in &self.types {
            writeln!(f, "// Internal index: {idx:?}\n{}\n", self.display(alias))?;
        }

        for (idx, symbol) in &self.symbols {
            writeln!(f, "// Internal index: {idx:?}\n{}\n", self.display(symbol))?;
        }

        Ok(())
    }
}

/// Symbol enum.
#[derive(Clone, PartialEq)]
pub enum Symbol {
    /// See [`crate::Function`].
    Function(crate::Function),
}

impl Symbol {
    /// Get the optional name of this symbol.
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Function(func) => func.name.as_deref(),
        }
    }

    /// Get the type of this symbol.
    pub fn ty(&self) -> crate::Type {
        match self {
            Self::Function(func) => func.ptr_type(),
        }
    }

    /// Get the type of this symbol.
    pub fn generics(&self) -> &[crate::TyParam] {
        match self {
            Self::Function(func) => &func.generics,
        }
    }
}

impl std::fmt::Display for crate::Display<'_, Symbol> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &**self {
            Symbol::Function(func) => self.project(func).fmt(f),
        }
    }
}

/// An ID of a symbol (function or global) inside of the module.
/// The frontend is in charge of providing unique IDs.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolId(pub u32);

impl From<usize> for SymbolId {
    fn from(value: usize) -> Self {
        Self(value as _)
    }
}

impl From<SymbolId> for usize {
    fn from(value: SymbolId) -> Self {
        value.0 as Self
    }
}

impl std::fmt::Debug for SymbolId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SymbolId#{}", self.0)
    }
}

impl std::fmt::Display for crate::Display<'_, SymbolId> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.module.symbols.get(self).map(|symbol| symbol.name()) {
            None => write!(f, "invalid#{}", self.0),
            Some(None) => write!(f, "unnamed#{}", self.0),
            Some(Some(name)) => write!(f, "{name}"),
        }
    }
}
