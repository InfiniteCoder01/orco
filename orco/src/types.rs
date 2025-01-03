use super::*;

/// Type. Can be a primitive or a whole struct
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
    /// Wildcard type
    #[default]
    Wildcard,

    /// Never type
    Never,
    /// Unit type (like C void)
    Unit,

    /// Signed integer type, size stored in bits
    Integer(u16),
    /// Unsigned integer type, size stored in bits
    Unsigned(u16),
    /// Floating point type, size stored in bits
    Float(u16),
    // Struct(Arc<RwLock<dyn Struct>>),
    // Tuple(M::Ref<'a, dyn Tuple>),
    // Reference(SymbolRef<dyn TypeTrait>),
}

#[debug_display]
impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wildcard => write!(f, "_"),

            Self::Never => write!(f, "!"),
            Self::Unit => write!(f, "()"),

            Self::Integer(size) => write!(f, "i{}", size),
            Self::Unsigned(size) => write!(f, "u{}", size),
            Self::Float(size) => write!(f, "f{}", size),
            // Self::Reference(r#type) => {
            //     write!(f, "f{}", r#type.handler().try_read().unwrap().name())
            // }
        }
    }
}
