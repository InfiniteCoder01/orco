use super::Symbol;

/// Type of a variable, constant, part of a function signature, etc.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
    /// Signed integer
    Integer(IntegerSize),
    /// Unsigned integer
    Unsigned(IntegerSize),
    /// IEEE (or not) floating point number with set number of bits
    Float(u16),
    /// A boolean. Should be 1 byte I guess...
    Bool,
    /// Just a symbol
    Symbol(Symbol),

    /// An array type (`Type[size]`)
    Array(Box<Type>, usize),
    /// A struct, aka a collection of field-type pairs.
    Struct {
        /// Struct fields
        fields: Vec<(Option<String>, Type)>,
    },
    /// Pointer (with mutability)
    Ptr(Box<Type>, bool),
    /// Function pointer
    FnPtr {
        /// Types of parameters
        params: Vec<Type>,
        /// Return type
        return_type: Option<Box<Type>>,
    },
    /// An error type, can also be used in a pointer to make it a pointer to anything
    Error,
}

impl Type {
    /// Returns a type name that could be used for hashing, mangling
    /// and human-facing names
    pub fn hashable_name(&self) -> String {
        match self {
            ty @ (Type::Integer(..) | Type::Unsigned(..) | Type::Float(..) | Type::Bool) => {
                ty.to_string()
            }

            Type::Symbol(sym) => sym.to_string(),
            Type::Array(ty, len) => format!("{}[{len}]", ty.hashable_name()),
            Type::Struct { fields } => fields
                .iter()
                .map(|(_, ty)| ty.hashable_name())
                .collect::<Vec<_>>()
                .join(" "),
            Type::Ptr(ty, mutable) => {
                format!(
                    "*{} {}",
                    match mutable {
                        true => "mut",
                        false => "const",
                    },
                    ty.hashable_name()
                )
            }
            Type::FnPtr {
                params,
                return_type,
            } => format!(
                "({}) -> {}",
                params
                    .iter()
                    .map(Type::hashable_name)
                    .collect::<Vec<_>>()
                    .join(", "),
                return_type
                    .as_deref()
                    .map_or("void".to_owned(), Type::hashable_name)
            ),
            Type::Error => "<error>".to_owned(),
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Integer(size) => write!(f, "i{size}"),
            Type::Unsigned(size) => write!(f, "u{size}"),
            Type::Float(size) => write!(f, "f{size}"),
            Type::Bool => write!(f, "bool"),

            Type::Symbol(sym) => write!(f, "{sym}"),
            Type::Array(ty, len) => write!(f, "{ty}[{len}]"),
            Type::Struct { fields } => {
                write!(f, "{{{}", if f.alternate() { '\n' } else { ' ' })?;
                for (idx, (name, ty)) in fields.iter().enumerate() {
                    if f.alternate() {
                        write!(f, "  ")?;
                    } else if idx > 0 {
                        write!(f, ", ")?;
                    }

                    match name {
                        Some(name) => write!(f, "{name}: ")?,
                        None => write!(f, "_{idx}: ")?,
                    }

                    ty.fmt(f)?;
                    if f.alternate() {
                        writeln!(f, ",")?;
                    }
                }

                write!(f, "{}}}", if f.alternate() { "" } else { " " })
            }
            Type::Ptr(ty, mutable) => {
                write!(
                    f,
                    "*{} ",
                    match mutable {
                        true => "mut",
                        false => "const",
                    },
                )?;
                ty.fmt(f)
            }
            Type::FnPtr {
                params,
                return_type,
            } => {
                write!(f, "(")?;

                for (idx, param) in params.iter().enumerate() {
                    if idx > 0 {
                        write!(f, ", ")?;
                    }

                    param.fmt(f)?;
                }

                match return_type {
                    Some(ty) => {
                        write!(f, ") -> ")?;
                        ty.fmt(f)
                    }
                    None => write!(f, ") -> void"),
                }
            }
            Type::Error => write!(f, "<error>"),
        }
    }
}

/// Integer size
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IntegerSize {
    /// Number of bits. Not sure if non-powers-of-two
    /// should be supported. Maybe even non-whole bytes (ex. u6 for 6 bit unsigned integer)
    Bits(u16),
    /// Kinda like `usize`/`isize` in rust or `size_t`/`ssize_t` in C
    Size,
}

impl std::fmt::Display for IntegerSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bits(bits) => write!(f, "{bits}"),
            Self::Size => write!(f, "size"),
        }
    }
}

/// Function signature without a name
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctionSignature {
    /// Parameter types with optional names
    pub params: Vec<(Option<String>, Type)>,
    /// Return type
    pub return_type: Option<Type>,
    /// Function attributes
    pub attrs: crate::attrs::FunctionAttributes,
}

impl FunctionSignature {
    #[allow(missing_docs)]
    pub fn new(
        params: Vec<(Option<String>, Type)>,
        return_type: Option<Type>,
        attrs: crate::attrs::FunctionAttributes,
    ) -> Self {
        Self {
            params,
            return_type,
            attrs,
        }
    }

    /// Get function pointer type for this function signature
    pub fn ptr_type(&self) -> Type {
        Type::FnPtr {
            params: self.params.iter().map(|(_, ty)| ty.clone()).collect(),
            return_type: self.return_type.clone().map(Box::new),
        }
    }
}

impl std::fmt::Display for FunctionSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;

        for (idx, (name, ty)) in self.params.iter().enumerate() {
            if idx > 0 {
                write!(f, ", ")?;
            }

            match name {
                Some(name) => write!(f, "{name:}: ")?,
                None => write!(f, "_{idx}: ")?,
            }

            ty.fmt(f)?;
        }

        match &self.return_type {
            Some(ty) => {
                write!(f, ") -> ")?;
                ty.fmt(f)
            }
            None => write!(f, ") -> void"),
        }
    }
}
