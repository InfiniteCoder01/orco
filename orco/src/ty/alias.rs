use super::Type;

/// A type alias, the way to declare types in orco.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Alias {
    /// Optional name.
    pub name: Option<String>,
    /// Type parameters.
    pub generics: Vec<super::TyParam>,
    /// Type to alias.
    pub ty: Type,
}

impl std::fmt::Display for crate::Display<'_, Alias> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "type {}{} = {};",
            self.name.as_deref().unwrap_or("(unnamed)"),
            self.project(&self.generics[..]),
            self.project(&self.ty),
        )
    }
}

/// An ID of a type alias inside of the module.
/// The frontend is in charge of providing unique IDs.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AliasId(pub u32);

impl From<usize> for AliasId {
    fn from(value: usize) -> Self {
        Self(value as _)
    }
}

impl From<AliasId> for usize {
    fn from(value: AliasId) -> Self {
        value.0 as Self
    }
}

impl std::fmt::Debug for AliasId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AliasId#{}", self.0)
    }
}

impl std::fmt::Display for crate::Display<'_, AliasId> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.module.types.get(self).map(|alias| &alias.name) {
            None => write!(f, "invalid#{}", self.0),
            Some(None) => write!(f, "unnamed#{}", self.0),
            Some(Some(name)) => write!(f, "{name}"),
        }
    }
}
