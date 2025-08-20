pub use stringleton::Symbol;

/// Type of a variable, constant, part of a function signature, etc.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
    /// Just a symbol
    Symbol(Symbol),
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
pub trait DefinitionBackend: PrimitiveTypeSource {
    /// Define a function, see [Codegen]
    fn define_function(&mut self, name: Symbol) -> impl Codegen<'_>;
}

/// A block label
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Label(pub usize);

/// An SSA value
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Value(pub usize);

/// Trait for generating code within a function
pub trait Codegen<'a> {
    /// See [PrimitiveTypeSource]
    type PTS: PrimitiveTypeSource;

    /// Return the primitive type source for this codegen,
    /// see [PrimitiveTypeSource] for more
    fn pts(&self) -> &Self::PTS;

    /// Get function parameter symbol by index
    fn param(&self, idx: usize) -> Symbol;

    /// Create an integer constant value
    fn iconst(&mut self, ty: Type, value: i128) -> Value;
    /// Create an unsigned integer constant value
    fn uconst(&mut self, ty: Type, value: u128) -> Value;

    /// Create a variable
    fn define_variable(&mut self, name: Symbol, ty: Type, mutable: bool, value: Option<Value>);
    /// Assign a value to variable
    fn assign_variable(&mut self, name: Symbol, value: Value);
    /// Access a variable (function parameters are also added as variables)
    fn variable(&mut self, symbol: Symbol) -> Value;
    /// Create a slot (aka an anonymous variable).
    /// Must be defined using [`Codegen::define_variable`]
    fn new_slot(&mut self) -> Symbol;

    /// Return a value from a function
    fn return_(&mut self, value: Option<Value>);

    /// If statement, immediately starts then block.
    /// Must be terminated with [`Codegen::end`]
    fn if_(&mut self, cond: Value);
    /// Add else block to the if statement and switch to it
    fn else_(&mut self);
    /// End current control flow construct
    fn end(&mut self);
}
