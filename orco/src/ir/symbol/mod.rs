use super::*;
use crate::symbol_reference::*;

/// A function
pub mod function;
pub use function::Function;

#[derive(Debug)]
/// A symbol
pub enum Symbol {
    /// A function
    Function(Function),
    /// External function
    ExternalFunction(ExternFunctionReference),
}

impl Symbol {
    /// Format
    pub fn format(&self, f: &mut std::fmt::Formatter<'_>, name: Option<&str>) -> std::fmt::Result {
        match self {
            Symbol::Function(function) => function.format(f, name),
            Symbol::ExternalFunction(function) => {
                write!(f, "extern ")?;
                function.format(f, name)?;
                write!(f, ";")?;
                Ok(())
            }
        }
    }

    /// Get a function signature of this symbol if it's is a function in any way (normal
    /// function, extern function, etc.)
    pub fn function_signature(&self) -> Option<&function::Signature> {
        match self {
            Symbol::Function(function) => Some(&function.signature),
            Symbol::ExternalFunction(signature) => Some(signature),
        }
    }
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f, None)
    }
}
