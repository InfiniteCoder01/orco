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
    /// Character, storing it's width (wide vs ascii)
    Char(bool),
    /// Points to a type alias, includes generics
    Symbol(Symbol, Vec<Type>),

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
    /// Type parameter (aka generic)
    Param(Symbol),
    /// An error type, can also be used in a pointer to make it a pointer to anything
    Error,
}

impl Type {
    /// Replace all instances of [`Type::Param`] with symbols from `map` (if present)
    pub fn instantiate(&mut self, map: &std::collections::HashMap<Symbol, Type>) {
        match self {
            Type::Integer(..)
            | Type::Unsigned(..)
            | Type::Float(..)
            | Type::Bool
            | Type::Char(..) => (),
            Type::Symbol(_, generics) => {
                for ty in generics {
                    ty.instantiate(map);
                }
            }
            Type::Array(ty, _) => ty.instantiate(map),
            Type::Struct { fields } => {
                for (_, ty) in fields {
                    ty.instantiate(map);
                }
            }
            Type::Ptr(ty, _) => {
                ty.instantiate(map);
            }
            Type::FnPtr {
                params,
                return_type,
            } => {
                for param in params {
                    param.instantiate(map);
                }
                if let Some(ty) = return_type {
                    ty.instantiate(map);
                }
            }
            Type::Param(name) => {
                if let Some(ty) = map.get(name) {
                    ty.clone_into(self);
                }
            }
            Type::Error => (),
        }
    }

    /// Same as [`Self::instantiate`], but clones the type in the process
    pub fn copy_instantiate(&self, map: &std::collections::HashMap<Symbol, Type>) -> Self {
        let mut instance = self.clone();
        instance.instantiate(map);
        instance
    }

    /// Check if this type contains type params
    pub fn has_params(&self) -> bool {
        match self {
            Type::Integer(..) => false,
            Type::Unsigned(..) => false,
            Type::Float(..) => false,
            Type::Bool => false,
            Type::Char(..) => false,
            Type::Symbol(_, generics) => {
                for generic in generics {
                    if generic.has_params() {
                        return true;
                    }
                }
                false
            }
            Type::Array(ty, _) => ty.has_params(),
            Type::Struct { fields } => {
                for (_, field) in fields {
                    if field.has_params() {
                        return true;
                    }
                }
                false
            }
            Type::Ptr(ty, _) => ty.has_params(),
            Type::FnPtr {
                params,
                return_type,
            } => {
                for param in params {
                    if param.has_params() {
                        return true;
                    }
                }
                return_type.as_deref().is_some_and(Type::has_params)
            }
            Type::Param(..) => true,
            Type::Error => false,
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
            Type::Char(false) => write!(f, "achar"),
            Type::Char(true) => write!(f, "uchar"),

            Type::Symbol(sym, generics) => write!(f, "{sym}{}", fmt_generics(generics)),
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
            Type::Param(name) => write!(f, "#{name}"),
            Type::Error => write!(f, "<error>"),
        }
    }
}

/// Format generic args using <> notation
pub fn fmt_generics(generics: &[Type]) -> String {
    if generics.is_empty() {
        return String::new();
    }

    let mut buffer = String::from("<");
    use std::fmt::Write as _;
    for (idx, ty) in generics.iter().enumerate() {
        if idx > 0 {
            buffer.push_str(", ");
        }
        write!(&mut buffer, "{ty}").unwrap();
    }
    buffer.push('>');
    buffer
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
    #[must_use]
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

    /// See [Type::instantiate]
    pub fn instantiate(&mut self, map: &std::collections::HashMap<Symbol, Type>) {
        for (_, ty) in &mut self.params {
            ty.instantiate(map);
        }
        if let Some(ty) = &mut self.return_type {
            ty.instantiate(map);
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
