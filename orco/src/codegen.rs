//! Code generation APIs, used to actually define functions and generate code.
use crate::{Symbol, Type};

/// A variable ID.
/// Variables are the only thing that can store information
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Variable(pub usize);

/// A variable ([Variable]) with projection (aka field access, dereferences, etc.)
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Place {
    /// Just variable access
    Variable(Variable),
    /// Pointer dereference
    Deref(Box<Place>),
    /// Field access
    Field(Box<Place>, Symbol),
}

/// Integer size
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IntegerSize {
    /// Number of bits
    Bits(u16),
    /// Kinda like `usize`/`isize` in rust or `size_t`/`ssize_t` in C
    Size,
}

/// An operand
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Operand {
    /// A global symbol, such as a function
    /// or a global variable/constant
    Global(Symbol),
    /// A place, see [Place]
    Place(Place),
    /// A signed integer constant
    IConst(i128, IntegerSize),
    /// An unsigned integer constant
    UConst(u128, IntegerSize),
    /// A floating point constant (value, size) where size is specified in bits
    FConst(f64, u16),
    /// Unit value
    Unit,
}

/// Trait for generating code within a function
pub trait BodyCodegen<'a> {
    /// The body generated is external
    fn external(self)
    where
        Self: Sized;

    /// Comment (doesn't have to be a comment from the original source code, could be a compiler comment)
    fn comment(&mut self, comment: &str);

    /// Declare a variable, see [Variable]
    fn declare_var(&mut self, ty: Type) -> Variable;
    /// Get the variable representing an argument
    fn arg_var(&self, idx: usize) -> Variable;

    /// Assign a value to a variable
    fn assign(&mut self, value: Operand, destination: Place);

    /// Call a function and put return value into `destination`
    fn call(&mut self, function: Operand, args: Vec<Operand>, destination: Place);

    /// Return a value from this function.
    /// Use [`Operand::Unit`] if no return value is required.
    fn return_(&mut self, value: Operand);
}
