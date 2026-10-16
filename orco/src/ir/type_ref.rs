use crate::Type;

/// Id of a type (index into list of used types).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeId(pub u32);

impl std::fmt::Display for TypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}

impl super::Body {
    /// Retrieve the [`Type`] by ID.
    pub fn interned_ty(&self, id: TypeId) -> &Type {
        &self
            .types
            .get(id.0)
            .unwrap_or_else(|| panic!("invalid type id {id}"))
    }

    /// Reference a symbol from the global namespace, adding it to the list of symbols
    /// (unless already there), returns the ID to be used with [`Self::symbol`].
    pub fn intern_ty(&mut self, ty: Type) -> TypeId {
        TypeId(self.types.insert(ty))
    }
}
