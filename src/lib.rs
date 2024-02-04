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

pub use codebase::{Codebase, FileId, Symbol};
pub use diagnostic::{Diagnostic, Label};

/// A compilation unit
pub trait Unit {
    /// Visit all items in this unit, building them in process
    fn visit_items(&self, codebase: &Codebase, visitor: &dyn Fn(&[Symbol], &ir::Item));
    /// Build a single item inside of this unit and return it's IR
    fn get_item(
        &self,
        others: &std::collections::HashMap<String, Box<dyn Unit>>,
        path: &[Symbol],
    ) -> &ir::Item;
}

impl Codebase {
    /// Parse a string path into a vector of symbols
    pub fn parse_path(&self, path: &str) -> Vec<Symbol> {
        path.split("::")
            .map(|segment| self.interned(segment))
            .collect()
    }

    /// Append a path to std::path::Path. Useful for module resolution
    pub fn append_path(&self, root: &std::path::Path, path: &[Symbol]) -> std::path::PathBuf {
        let mut root = root.to_path_buf();
        for segment in path {
            root.push(self.resolve_symbol(*segment));
        }
        root
    }
}
