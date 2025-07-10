pub use stringleton::Symbol;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
    Symbol(Symbol),
}

pub trait TypeBackend {
    /// Get a unit type instance (aka C `void` or Rust `()`)
    fn unit(&self) -> Symbol;

    /// Get an integer type instance
    fn int(&self, size: u16, signedness: bool) -> Symbol;

    /// Get a size_t kind integer type instance
    fn size_type(&self, signedness: bool) -> Symbol;
}

/// Root trait for declaring module items, that includes full types.
pub trait DeclarationBackend: TypeBackend {
    /// Declare a function
    fn function(&mut self, name: Symbol, params: &[(Option<Symbol>, Type)], return_type: &Type);
}
