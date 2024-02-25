use super::*;

/// A function
pub mod function;
pub use function::Function;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// An item
pub enum Item {
    /// A function
    Fn(Function),
}
