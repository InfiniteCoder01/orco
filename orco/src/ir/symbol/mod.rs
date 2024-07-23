use super::*;

/// A function
pub mod function;
pub use function::Function;

use std::sync::Arc;

#[derive(Debug)]
/// A symbol
pub enum Symbol {
    /// A comptime assignment
    Comptime(PathSegment, Expression),
    /// A function
    Function(Arc<ir::symbol::Function>),
    /// External function
    ExternalFunction(Arc<ir::symbol::function::Signature>),
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Symbol::Comptime(name, value) => write!(f, "comptime {} = {};", name, value),
            Symbol::Function(function) => function.fmt(f),
            Symbol::ExternalFunction(function) => {
                write!(f, "extern {};", function)
            }
        }
    }
}
