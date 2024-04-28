use crate::TypeVariableID;
use std::num::NonZeroU16;

/// A type enum consists of all builtin types and a custom variant
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
    /// Integer, f.e. i32, i64, etc. Stores the size in bytes
    Int(NonZeroU16),
    /// Unsigned integer, f.e. u8, u32, u64, etc. Stores the size in bytes
    Unsigned(NonZeroU16),
    /// Floating point number, f.e. f32, f64. Stores the size in bytes
    Float(NonZeroU16),
    /// Boolean (true or false)
    Bool,
    /// Character
    Char,

    /// Pointer type
    Pointer(Box<Type>),
    /// Custom type, f.e. a struct or a type alias
    Custom(String),

    /// Never type, can't hold any value
    Never,

    /// Unit type, can only hold one value
    Unit,

    /// A wildcard type (aka non-inferred)
    Wildcard,
    /// Type variable (used only during type inference)
    TypeVariable(TypeVariableID),
    /// Error type
    Error,
}

impl Type {
    /// Create a new unit type
    pub fn unit() -> Self {
        Self::Unit
    }

    /// Does this type morphs to the target type
    pub fn morphs(&self, target_type: &Type) -> bool {
        self == target_type
            || self == &Type::Never
            || self == &Type::Wildcard
            || self == &Type::Error
            || target_type == &Type::Never
            || target_type == &Type::Wildcard
            || target_type == &Type::Error
    }

    /// Is this type complete (nothing to infer)
    /// TypeVariables are considered complete
    pub fn complete(&self) -> bool {
        !matches!(self, Self::Wildcard | Self::Error)
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(size) => write!(f, "i{}", size.get() * 8),
            Self::Unsigned(size) => write!(f, "u{}", size.get() * 8),
            Self::Float(size) => write!(f, "f{}", size.get() * 8),
            Self::Bool => write!(f, "bool"),
            Self::Char => write!(f, "char"),
            Self::Never => write!(f, "!"),
            Self::Pointer(r#type) => write!(f, "{}*", r#type),
            Self::Unit => write!(f, "()"),
            Self::Custom(name) => write!(f, "{}", name),
            Self::Wildcard => write!(f, "_"),
            Self::TypeVariable(id) => write!(f, "{}", id),
            Self::Error => write!(f, "<ERROR>"),
        }
    }
}

impl std::ops::BitOrAssign for Type {
    fn bitor_assign(&mut self, rhs: Type) {
        match self {
            Self::Never | Self::Error | Self::Wildcard => *self = rhs,
            _ => (),
        }
    }
}

impl std::ops::BitOr for Type {
    type Output = Type;

    fn bitor(mut self, rhs: Type) -> Self::Output {
        self |= rhs;
        self
    }
}
