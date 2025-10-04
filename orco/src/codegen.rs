use crate::{Symbol, Type};

/// A variable ID.
/// Variables are the only thing that can store information
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Variable(pub usize);

/// An operand
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Operand {
    /// A global symbol, such as a function
    /// or a global variable/constant
    Global(Symbol),
    /// A variable, see [Variable]
    Variable(Variable),
    /// A signed integer constant
    IConst(i128), // TODO: size
    /// An unsigned integer constant
    UConst(u128), // TODO: size
    /// Unit value
    Unit,
}

/// Trait for generating code within a function
pub trait Codegen<'a> {
    /// Comment (doesn't have to be a comment from the original source code, could be a compiler comment)
    fn comment(&mut self, comment: &str);

    /// Declare a variable, see [Variable]
    fn declare_var(&mut self, ty: &Type) -> Variable;
    /// Get the variable representing an argument
    fn arg_var(&self, idx: usize) -> Variable;

    // TODO
    /// Cast a value to `destination`'s type.
    /// Bascially any standard integer-float-char-bool cast.
    /// Might be removed in favor of [`Codegen::call`] & intrinsics
    fn cast(&mut self, value: Operand, destination: Variable);

    /// Call a function and put return value into `destination`
    fn call(&mut self, function: Operand, args: Vec<Operand>, destination: Variable);

    /// Return a value from this function.
    /// Use [`Operand::Unit`] if no return value is required.
    fn return_(&mut self, value: Operand);
}
