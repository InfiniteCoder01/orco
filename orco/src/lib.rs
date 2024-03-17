#![warn(missing_docs)]
//! OrCo is the base crate for OrCo compiler toolchain.
//! It's used in parser and backend crates as a glue.

/// Intermediate Representation lives here
pub mod ir;

/// A compilation unit
pub trait Unit {
    // /// Build all items in this unit, visiting them one by one
    // fn visit_items(&self, visitor: &mut dyn FnMut(&[Symbol], &ir::Item));
}
