#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

pub use orco_procmacro::*;

/// Symbols are essential parts of the program, like functions,
/// constants, static variables, thread locals, types, macros, etc.
pub mod symbol;
pub use symbol::Symbol;

/// Symbol references are one of the key features of OrCo.
/// They allow symbols to be accessed from anywhere
pub mod symbol_box;
pub use symbol_box::{SymbolBox, SymbolRef};

/// Boxed dynamic iterator
pub type DynIter<'a, T> = Box<dyn Iterator<Item = T> + 'a>;

// /// A single unit just houses symbols
// #[debug_display]
// pub trait Unit {
//     /// Returns a dynamic iterator over all symbols in this unit
//     fn symbols<'a>(&'a self) -> DynIter<'a, &'a Symbol>;
//     /// Returns a mutable dynamic iterator over all symbols in this unit
//     fn symbols_mut<'a>(&'a mut self) -> DynIter<'a, &'a mut Symbol>;
// }

// impl std::fmt::Display for dyn Unit {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         for symbol in self.symbols() {
//             writeln!(f, "{}", symbol)?;
//         }
//         Ok(())
//     }
// }
