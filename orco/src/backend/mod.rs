pub use stringleton::Symbol;

/// Type of a variable, constant, part of a function signature, etc.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
    /// Just a symbol
    Symbol(Symbol),
}

/// A backend that can generate types.
/// Every method here returns an orco [Type] that will
/// be used in codegen.
pub trait TypeBackend {
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

/// Root trait for declaring module items, that includes full types.
pub trait DeclarationBackend: TypeBackend {
    /// Declare a function
    fn function(&mut self, name: Symbol, params: &[(Option<Symbol>, Type)], return_type: &Type);
}
