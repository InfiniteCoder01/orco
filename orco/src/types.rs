use super::*;

/// Type. Can be a primitive or a whole struct
#[derive(Clone, Default)]
pub enum Type {
    /// Wildcard type
    #[default]
    Wildcard,

    /// Never type
    Never,
    /// Unit type (like C void)
    Unit,

    /// Boolean type
    Bool,
    /// Signed integer type, size stored in bits
    Integer(u16),
    /// Unsigned integer type, size stored in bits
    Unsigned(u16),
    /// Floating point type, size stored in bits
    Float(u16),

    /// Function if const, function pointer otherwise
    Fn(FunctionSignature),

    /// Type that hasn't been resolved yet
    Unresolved(String),
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

            Self::Bool => write!(f, "bool"),
            Self::Integer(size) => write!(f, "i{}", size),
            Self::Unsigned(size) => write!(f, "u{}", size),
            Self::Float(size) => write!(f, "f{}", size),

            Self::Fn(signature) => write!(f, "fn {}", signature),

            Self::Unresolved(name) => write!(f, "'{}'", name),
            // Self::Reference(r#type) => {
            //     write!(f, "f{}", r#type.handler().read().unwrap().name())
            // }
        }
    }
}

#[macro_export]
macro_rules! quote_type {
    (_) => {
        $crate::types::Type::Wildcard
    };
    (!) => {
        $crate::types::Type::Never
    };
    (()) => {
        $crate::types::Type::Unit
    };
    (bool) => {
        $crate::types::Type::Bool
    };
    (u8) => {
        $crate::types::Type::Unsigned(8)
    };
    (u16) => {
        $crate::types::Type::Unsigned(16)
    };
    (u32) => {
        $crate::types::Type::Unsigned(32)
    };
    (u64) => {
        $crate::types::Type::Unsigned(64)
    };
    (i8) => {
        $crate::types::Type::Signed(8)
    };
    (i16) => {
        $crate::types::Type::Signed(16)
    };
    (i32) => {
        $crate::types::Type::Signed(32)
    };
    (i64) => {
        $crate::types::Type::Signed(64)
    };
    (i8) => {
        $crate::types::Type::Float(8)
    };
    (i16) => {
        $crate::types::Type::Float(16)
    };
    (i32) => {
        $crate::types::Type::Float(32)
    };
    (i64) => {
        $crate::types::Type::Float(64)
    };
    ((fn $args: tt -> $return: tt)) => {
        $crate::types::Type::Fn($crate::function_signature![$args -> $return])
    };
    ({$ty: expr}) => {
        $ty
    };
    ($ty: literal) => {
        $crate::types::Type::Unresolved($ty)
    };
}

/// Function signature. Contains all the typing information about this function
#[derive(Clone)]
pub struct FunctionSignature {
    /// Function parameters, optional names and types
    pub parameters: Vec<(Option<String>, Type)>,
    /// Return type of the function
    pub return_type: Box<Type>,
}

impl FunctionSignature {
    pub fn new(parameters: Vec<(Option<String>, Type)>, return_type: Type) -> Self {
        Self {
            parameters,
            return_type: Box::new(return_type),
        }
    }
}

impl std::fmt::Display for FunctionSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(",)?;
        for (index, (name, r#type)) in self.parameters.iter().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }
            write!(
                f,
                "{}: {}",
                name.as_ref().map_or("_", String::as_str),
                r#type
            );
        }
        write!(f, ") -> {}", self.return_type)?;
        Ok(())
    }
}

#[macro_export]
macro_rules! function_signature {
    (($($name:ident: $ty:tt),*) -> $rt:tt) => {
        $crate::types::FunctionSignature::new(vec![$((Some(stringify!($name).to_owned()), $crate::quote_type![$ty])),*], $crate::quote_type![$rt])
    }
}
