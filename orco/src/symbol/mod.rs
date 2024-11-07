use super::*;

/// Function is where all the code is
pub mod function;
pub use function::Function;

/// Symbols are essential parts of the program, like functions,
/// constants, static variables, thread locals, types, macros, etc.
#[derive(Clone, Copy)]
pub enum Symbol<'a> {
    /// Function definition
    Function(&'a std::sync::RwLock<dyn Function + 'a>),
}

#[debug_display]
impl std::fmt::Display for Symbol<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Function(function) => (&*function.try_read().unwrap()).fmt(f),
        }
    }
}
