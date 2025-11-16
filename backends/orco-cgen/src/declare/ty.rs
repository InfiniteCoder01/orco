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

impl crate::Backend {
    /// Convert [`orco::Type`] to [Type]
    pub fn convert_type(&self, ty: &orco::Type) -> Type {
        use orco::Type as OT;
        match ty {
            OT::Symbol(sym) => Type(crate::escape(*sym)),
            OT::Array(ty, size) => todo!(),
            OT::Struct(fields) => todo!(),
            OT::Error => Type::error(),
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
