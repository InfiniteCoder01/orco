/// Literally just a wrapper around a string for holding a type in C land
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Type(String);

impl Type {
    /// Construct an error type, should not appear in the source code
    pub fn error() -> Self {
        Self("<error-type>".to_owned())
    }

    /// Check if the type is "void"
    pub fn is_void(&self) -> bool {
        self.0 == "void"
    }
}

impl Default for Type {
    fn default() -> Self {
        Self::error()
    }
}

impl From<&orco::Type> for Type {
    fn from(ty: &orco::Type) -> Self {
        use orco::Type;
        match ty {
            Type::Symbol(sym) => Self(sym.to_string()),
            Type::Array(ty, size) => Self(format!("{ty}[{size}]", ty = Self::from(ty.as_ref()))),
            Type::Error => Self::error(),
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
