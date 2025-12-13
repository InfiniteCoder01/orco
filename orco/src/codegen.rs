//! Code generation APIs, used to actually define functions and generate code.
use crate::{Symbol, Type};

/// A variable ID.
/// Variables are the only thing that can store information
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Variable(pub usize);

/// A variable or symbol with projection (aka field access, dereferences, etc.)
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Place {
    /// Just variable access
    Variable(Variable),
    /// Global symbol access
    Global(Symbol),
    /// Pointer dereference
    Deref(Box<Place>),
    /// Field access
    Field(Box<Place>, Symbol),
}

/// An operand
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Operand {
    /// A place, see [Place]
    Place(Place),
    /// A signed integer constant
    IConst(i128, crate::IntegerSize),
    /// An unsigned integer constant
    UConst(u128, crate::IntegerSize),
    /// A floating point constant (value, size) where size is specified in bits
    FConst(f64, u16),
    /// Unit value
    Unit,
}

/// A label ID. See [`BodyCodegen::label`]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Label(pub usize);

/// Trait for generating code within a function
pub trait BodyCodegen<'a> {
    /// The body generated is external
    fn external(self)
    where
        Self: Sized;

    /// Declare a variable, see [Variable]
    fn declare_var(&mut self, ty: Type) -> Variable;
    /// Get the variable representing an argument
    fn arg_var(&self, idx: usize) -> Variable;

    /// Assign a value to a variable
    fn assign(&mut self, value: Operand, destination: Place);

    /// Call a function and put return value into `destination`
    fn call(&mut self, function: Operand, args: Vec<Operand>, destination: Place);

    /// Puts a said label in the current position.
    /// Note: Labels can be used before placing. Frontend decides on IDs
    /// Warning: Not all codegens implement arbitrary control flow
    fn label(&mut self, label: Label) {
        let _ = label;
        unimplemented!("arbitrary control flow is not supported by this backend");
    }

    /// Jump to a label
    /// See [`BodyCodegen::label`]
    fn jump(&mut self, label: Label) {
        let _ = label;
        unimplemented!("arbitrary control flow is not supported by this backend");
    }

    /// Return a value from this function.
    /// Use [`Operand::Unit`] if no return value is required.
    fn return_(&mut self, value: Operand);
}
