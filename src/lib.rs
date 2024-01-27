//! OrCo is the base crate for OrCo compiler toolchain.
//! It's used in parser and backend crates as a glue.
//! See [Codebase] to get started.

#![warn(missing_docs)]

/// Codebase lives here
pub mod codebase;
/// An interface for [codespan-reporting](https://crates.io/crates/codespan-reporting)
pub mod diagnostic;
/// Intermediate Representation lives here
pub mod ir;

pub use codebase::{Codebase, Symbol};
pub use diagnostic::{Diagnostic, Label, Severity};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// An input for the parser
pub enum Input<'a> {
    /// Take source code from the file
    File(&'a std::path::Path),
    /// Take source code from the string
    String(&'a str),
}

/// A compilation unit
pub trait Unit {
    /// Parse the unit from the input
    fn parse(input: Input) -> Self
    where
        Self: Sized;

    /// Build the IR from this unit
    fn build(self, codebase: &Codebase) -> ir::Module;
    /// Build a single item inside of this unit and return it's IR
    fn get_item(
        &self,
        others: &std::collections::HashMap<String, Box<dyn Unit>>,
        path: &[Symbol],
    ) -> &ir::Item;
}

impl Codebase<'_> {
    /// Parse a string path into a vector of symbols
    pub fn parse_path(&mut self, path: &str) -> Vec<Symbol> {
        path.split("::")
            .map(|segment| self.interner.get_or_intern(segment))
            .collect()
    }
}
