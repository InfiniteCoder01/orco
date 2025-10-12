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
            OT::Symbol(sym) => Type(sym.to_string()),
            OT::Array(ty, size) => {
                let ty = self.convert_type(ty);
                let symbol = format!("array_{ty}_{size}");
                let _ = self.arrays.insert_sync((ty, *size));
                Type(symbol)
            }
            OT::Error => Type::error(),
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
