#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

/// Diagnostics
pub mod diagnostics;
/// OrCo Intermediate Representation lives here
pub mod ir;
/// Source and span
pub mod source;
/// Type inference structs and functions
pub mod type_inference;
/// Variable maker
pub mod variable_mapper;

pub use source::*;
pub use type_inference::TypeInference;
