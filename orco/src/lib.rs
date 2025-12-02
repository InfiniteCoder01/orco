#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

pub use sinter;
pub use sinter::IStr as Symbol;

/// Code generation, outside of declaration
pub mod codegen;
pub use codegen::BodyCodegen;

// Utils for making implementations
pub mod type_intern;

/// Type of a variable, constant, part of a function signature, etc.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
    /// Just a symbol
    Symbol(Symbol),
    /// An array type (`Type[size]`)
    Array(Box<Type>, usize),
    /// A struct, aka a collection of field-type pairs
    Struct(Vec<(Symbol, Type)>),
    /// An error type
    Error,
}

impl Type {
    /// Returns a type name that could be used for hashing, mangling
    /// and human-facing names
    pub fn hashable_name(&self) -> String {
        match self {
            Type::Symbol(sym) => sym.to_string(),
            Type::Array(ty, len) => format!("{}[{len}]", ty.hashable_name()),
            Type::Struct(fields) => fields
                .iter()
                .map(|(_, ty)| ty.hashable_name())
                .collect::<Vec<_>>()
                .join(" "),
            Type::Error => "<error>".to_owned(),
        }
    }
}

/// Integer size
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IntegerSize {
    /// Number of bits
    Bits(u16),
    /// Kinda like `usize`/`isize` in rust or `size_t`/`ssize_t` in C
    Size,
}

/// This is a way to get primitive types. Every method returns an
/// orco [Type] that will be used by frontends
pub trait PrimitiveTypeSource {
    /// Get the boolean type
    fn bool(&self) -> Type;

    /// Get the integer type with the set size and signedness
    fn int(&self, size: IntegerSize, signed: bool) -> Type;

    /// Get the floating point type
    fn float(&self, size: u16) -> Type;
}

/// Root trait for declaring module items. This is enough to generate C headers
pub trait Backend: PrimitiveTypeSource + Sync {
    /// Declare a function
    fn function(
        &self,
        name: Symbol,
        params: Vec<(Option<Symbol>, Type)>,
        return_type: Type,
    ) -> impl codegen::BodyCodegen<'_>;

    /// Define a type alias, should be used to declare compound types as well
    fn type_(&self, name: Symbol, ty: Type);
}
