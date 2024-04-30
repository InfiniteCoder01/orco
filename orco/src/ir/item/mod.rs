use super::*;

/// A function
pub mod function;
pub use function::Function;

#[derive(Clone, Debug)]
/// An item
pub enum Item {
    /// A function
    Function(function::Function),
    /// External function
    ExternalFunction(function::Signature),
}

impl Item {
    /// Format
    pub fn format(&self, f: &mut std::fmt::Formatter<'_>, name: Option<&str>) -> std::fmt::Result {
        match self {
            Item::Function(function) => function.format(f, name),
            Item::ExternalFunction(function) => {
                write!(f, "extern ")?;
                function.format(f, name)?;
                write!(f, ";")?;
                Ok(())
            }
        }
    }

    /// Get a function signature of this item if this item is a function in any way (normal
    /// function, extern function, etc.)
    pub fn function_signature(&self) -> Option<&function::Signature> {
        match self {
            Item::Function(function) => Some(&function.signature),
            Item::ExternalFunction(signature) => Some(signature),
        }
    }
}

impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f, None)
    }
}
