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
    /// See [Codegen]
    type FunctionCodegen<'a>: FunctionCodegen<'a>
    where
        Self: 'a;

    /// Define a function
    fn define_function(&mut self, name: Symbol) -> Self::FunctionCodegen<'_>;
}

/// A unique label ID
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Label(usize);

/// Trait for generating code within a function
pub trait FunctionCodegen<'a> {
    /// See [PrimitiveTypeSource]
    type PTS: PrimitiveTypeSource;
    /// A value of an operation
    type Value: Copy;

    /// Return the primitive type source for this codegen,
    /// see [PrimitiveTypeSource] for more
    fn pts(&self) -> &Self::PTS;

    /// Get function parameter symbol by index
    fn param(&self, idx: usize) -> Symbol;

    /// Create an integer constant value
    fn iconst(&mut self, ty: Type, value: i128) -> Self::Value;
    /// Create an unsigned integer constant value
    fn uconst(&mut self, ty: Type, value: u128) -> Self::Value;

    /// Create a variable
    fn define_variable(
        &mut self,
        name: Symbol,
        ty: Type,
        mutable: bool,
        value: Option<Self::Value>,
    );
    /// Access a variable (function parameters are also added as variables)
    fn variable(&mut self, symbol: Symbol) -> Self::Value;

    /// Return a value from a function
    fn return_(&mut self, value: Option<Self::Value>);

    /// Create a new label
    fn new_label(&mut self) -> usize;
    /// Place the label into the program,
    /// can only be used ONCE per label
    fn label(&mut self, label: Label);
    /// Jump to label
    fn jump(&mut self, label: Label);
}
