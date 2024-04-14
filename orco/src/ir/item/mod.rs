use super::*;

/// A function
pub mod function;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// An item
pub enum Item {
    /// A function
    Function(function::Function),
}
