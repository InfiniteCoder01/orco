#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

pub use sinter;
pub use sinter::IStr as Symbol;

/// Code generation, outside of declaration
pub mod codegen;
pub use codegen::BodyCodegen;

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

/// This is a way to get primitive types. Every method returns an
/// orco [Type] that will be used by frontends
pub trait PrimitiveTypeSource {
    /// Get the unit type (aka C `void` or Rust `()`)
    fn unit(&self) -> Type;

    /// Get the boolean type
    fn bool(&self) -> Type;

    /// Get the integer type with the set size and signedness
    fn int(&self, size: u16, signedness: bool) -> Type;

    /// Get the size_t/usize kind integer type
    fn size_type(&self, signedness: bool) -> Type;

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
