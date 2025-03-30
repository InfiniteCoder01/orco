#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

/// Diagnostic and source span handling
pub mod diagnostic;
pub use diagnostic::DiagCtx;

/// OrCo backend interfaces
pub mod backend;
pub use backend::Backend;

/// Registery and everything that can be registered
pub mod registry;
pub use registry::{FunctionId, Parameter, Registry, Signature, Symbol, Type};
