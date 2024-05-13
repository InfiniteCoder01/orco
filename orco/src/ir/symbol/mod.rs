use super::*;
use crate::symbol_reference::*;

/// A function
pub mod function;
pub use function::Function;

#[derive(Debug)]
/// A symbol
pub enum Symbol {
    /// A function
    Function(FunctionReference),
    /// External function
    ExternalFunction(ExternFunctionReference),
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Symbol::Function(function) => function.fmt(f),
            Symbol::ExternalFunction(function) => {
                write!(f, "extern {};", function.inner)
            }
        }
    }
}
