#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Type(String);

impl Type {
    pub fn error() -> Self {
        Self("<error-type>".to_owned())
    }

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
            Type::Error => Self::error(),
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
