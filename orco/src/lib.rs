#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

/// Diagnostics
pub mod diagnostics;
/// Intermediate Representation lives here
pub mod ir;
/// Source
pub mod source;
/// Type inference
pub mod type_inference;
/// Variable maker
pub mod variable_mapper;

pub use source::*;
