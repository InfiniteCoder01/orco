#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

/// Diagnostics
pub mod diagnostics;
/// OrCo Intermediate Representation lives here
pub mod ir;
/// Source and span
pub mod source;
/// Symbol reference (used to reference symbols/variables in expressions)
pub mod symbol_reference;
/// Type inference structs and functions
pub mod type_inference;

pub use source::*;
pub use symbol_reference::SymbolReference;
pub use type_inference::TypeInference;
