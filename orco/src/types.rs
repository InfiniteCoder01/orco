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
    ((fn $args:tt -> $return:tt $($calling_conv:ident)?)) => {
        $crate::types::Type::Fn($crate::function_signature![$args -> $return $($calling_conv)?])
    };
    ({$ty:expr}) => {
        $ty
    };
    ($ty:literal) => {
        $crate::types::Type::Unresolved($ty)
    };
}

/// Function calling conventions
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CallingConvention {
    /// Basically a block of code, doesn't even have it's own stack frame
    Transparent,
    /// Inline the function
    Inline,
    /// Fastest, but ABI is unstable
    #[default]
    Fastest,
    /// System V calling convention (== cdecl)
    SystemV,
    /// fastcall
    Fastcall,
}

impl std::fmt::Display for CallingConvention {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transparent => write!(f, "transparent"),
            Self::Inline => write!(f, "inline"),
            Self::Fastest => write!(f, "fastest"),
            Self::SystemV => write!(f, "sys_v"),
            Self::Fastcall => write!(f, "fastcall"),
        }
    }
}

impl std::str::FromStr for CallingConvention {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "transparent" => Ok(Self::Transparent),
            "inline" => Ok(Self::Inline),
            "fastest" => Ok(Self::Fastest),
            "sys_v" => Ok(Self::SystemV),
            "fastcall" => Ok(Self::Fastcall),
            "default" => Ok(Self::default()),
            _ => Err(()),
        }
    }
}

/// Function signature. Contains all the typing information about this function
#[derive(Clone)]
pub struct FunctionSignature {
    /// Function parameters, optional names and types
    pub parameters: Vec<(Option<String>, Type)>,
    /// Return type of the function
    pub return_type: Box<Type>,
    /// Calling convention
    pub calling_convention: CallingConvention,
}

impl FunctionSignature {
    pub fn new(
        parameters: Vec<(Option<String>, Type)>,
        return_type: Type,
        calling_convention: CallingConvention,
    ) -> Self {
        Self {
            parameters,
            return_type: Box::new(return_type),
            calling_convention,
        }
    }
}

impl std::fmt::Display for FunctionSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        for (index, (name, r#type)) in self.parameters.iter().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }
            write!(
                f,
                "{}: {}",
                name.as_ref().map_or("_", String::as_str),
                r#type
            )?;
        }
        write!(f, ") -> {} {}", self.return_type, self.calling_convention)?;
        Ok(())
    }
}

#[macro_export]
macro_rules! function_signature {
    (($($name:ident: $ty:tt),*) -> $rt:tt $calling_convention:ident) => {
        $crate::types::FunctionSignature::new(
            vec![$((Some(stringify!($name).to_owned()), $crate::quote_type![$ty])),*],
            $crate::quote_type![$rt],
            <$crate::types::CallingConvention as std::str::FromStr>::from_str(stringify!($calling_convention)).unwrap()
        )
    };
    (($($name:ident: $ty:tt),*) -> $rt:tt) => {
        $crate::function_signature![($($name: $ty),*) -> $rt default]
    };
}
