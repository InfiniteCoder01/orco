#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

pub use sinter;
pub use sinter::IStr as Symbol;

pub mod codegen;

/// Type of a variable, constant, part of a function signature, etc.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
    /// Just a symbol
    Symbol(Symbol),
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
pub trait DeclarationBackend: PrimitiveTypeSource {
    /// Declare a function
    fn declare_function(
        &mut self,
        name: Symbol,
        params: &[(Option<Symbol>, Type)],
        return_type: &Type,
    );
}

/// Root trait for defining module items
pub trait DefinitionBackend: PrimitiveTypeSource + Sync {
    /// Define a function, see [Codegen]
    fn define_function(&self, name: Symbol) -> impl codegen::Codegen<'_>;
}
