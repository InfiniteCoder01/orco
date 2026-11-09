use thin_vec::ThinVec;

mod alias;
pub use alias::{Alias, AliasId};

mod generics;
pub use generics::TyParam;

/// Type of a variable, constant, part of a function signature, etc.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
    /// Unit type, aka C `void`.
    Unit,
    /// Signed integer.
    Integer(IntegerSize),
    /// Unsigned integer.
    Unsigned(IntegerSize),
    /// IEEE (or not) floating point number with set number of bits.
    Float(u8),
    /// A boolean. Should be 1 byte I guess...
    Bool,
    /// Character, storing it's width (wide vs ascii).
    Char(bool),
    /// Points to a type alias, includes generics.
    Alias(AliasId, ThinVec<Type>),

    /// An array type (`Type[size]`).
    Array(Box<Type>, u32),
    /// A struct, aka a collection of field-type pairs.
    Struct {
        /// Struct fields.
        fields: ThinVec<(Option<String>, Type)>,
    },
    /// Pointer (with mutability).
    Ptr(Box<Type>, bool),
    /// Function pointer ().
    FnPtr {
        /// Return type, followed by types of parameters.
        types: ThinVec<Type>,
    },

    /// Type parameter (aka generic).
    Param(u32),
    /// An error type, can also be used in a pointer to make it a pointer to anything.
    Error,
}

impl std::fmt::Debug for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unit => write!(f, "unit"),
            Self::Integer(size) => write!(f, "i{size}"),
            Self::Unsigned(size) => write!(f, "u{size}"),
            Self::Float(size) => write!(f, "f{size}"),
            Self::Bool => write!(f, "bool"),
            Self::Char(false) => write!(f, "achar"),
            Self::Char(true) => write!(f, "uchar"),

            Self::Alias(alias, generics) => write!(f, "{alias:?}{generics:?}"),
            Self::Array(ty, len) => write!(f, "{ty:?}[{len}]"),
            Self::Struct { fields } => {
                let mut dbg = f.debug_struct("struct");
                for (idx, (name, ty)) in fields.iter().enumerate() {
                    match name {
                        Some(name) => dbg.field(name, ty),
                        None => dbg.field(&format!("_{idx}"), ty),
                    };
                }
                dbg.finish()
            }
            Self::Ptr(ty, mutable) => {
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
            Self::FnPtr { types } => {
                write!(f, "(")?;

                for (idx, param) in types.iter().skip(1).enumerate() {
                    if idx > 0 {
                        write!(f, ", ")?;
                    }

                    param.fmt(f)?;
                }

                write!(f, ") -> ")?;
                types[0].fmt(f)
            }
            Self::Param(idx) => write!(f, "#{idx}"),
            Self::Error => write!(f, "<error>"),
        }
    }
}

impl std::fmt::Display for crate::Display<'_, Type> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &**self {
            Type::Alias(alias, generics) => {
                self.project(alias).fmt(f)?;
                self.project(&generics[..]).fmt(f)
            }
            Type::Array(ty, len) => write!(f, "{}[{len}]", self.project(ty.as_ref())),
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
                        None if f.alternate() => write!(f, "_{idx}: ")?,
                        None => (),
                    }

                    self.project(ty).fmt(f)?;
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
                self.project(ty.as_ref()).fmt(f)
            }
            Type::FnPtr { types } => {
                write!(f, "(")?;

                for (idx, param) in types.iter().skip(1).enumerate() {
                    if idx > 0 {
                        write!(f, ", ")?;
                    }

                    self.project(param).fmt(f)?;
                }

                write!(f, ") -> ")?;
                self.project(&types[0]).fmt(f)
            }

            ty => write!(f, "{ty:?}"),
        }
    }
}

/// Integer size
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IntegerSize {
    /// Kinda like `usize`/`isize` in rust or `size_t`/`ssize_t` in C.
    Size,
    /// Number of bits. Not sure if non-powers-of-two
    /// should be supported. Maybe even non-whole bytes (ex. u6 for 6 bit unsigned integer).
    Bits(std::num::NonZeroU8),
}

impl std::fmt::Display for IntegerSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Size => write!(f, "size"),
            Self::Bits(bits) => write!(f, "{bits}"),
        }
    }
}

impl crate::Function {
    /// Get function pointer type for this function signature
    pub fn ptr_type(&self) -> Type {
        Type::FnPtr {
            types: std::iter::once(self.return_type.clone())
                .chain(self.params.iter().map(|(_, ty)| ty.clone()))
                .collect(),
        }
    }
}
