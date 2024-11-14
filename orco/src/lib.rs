#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

pub use orco_procmacro::*;

/// See [Expression]
pub mod expression;
pub use expression::Expression;

/// See [Symbol]
pub mod symbol;
pub use symbol::Symbol;

/// See [Type]
pub mod types;
pub use types::Type;

/// Mutability
pub mod mutability;
pub use mutability::*;

/// Symbol references are one of the key features of OrCo.
/// They allow symbols to be accessed from anywhere
pub mod symbol_box;
pub use symbol_box::{SymbolBox, SymbolRef};

/// Boxed dynamic iterator
pub type DynIter<'a, T> = Box<dyn Iterator<Item = T> + 'a>;

/// A single unit just houses symbols
pub trait Unit {
    /// Returns a dynamic iterator over all symbols in this unit
    fn symbols(&self) -> DynIter<Symbol>;
}

#[debug_display]
impl std::fmt::Display for &dyn Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for symbol in self.symbols() {
            writeln!(f, "{}", symbol)?;
        }
        Ok(())
    }
}
