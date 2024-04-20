//! OrCo is the base crate for OrCo compiler toolchain.
//! It's used in parser and backend crates as a glue.
#![warn(missing_docs)]

/// Intermediate Representation lives here
pub mod ir;

/// Type inference information for a function
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeInferenceInfo<'a> {
    /// Module
    pub module: &'a ir::Module,
    /// Return type of a function
    pub return_type: &'a ir::Type,
}
