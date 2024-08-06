#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

pub use log;
pub use miette;
pub use thiserror;

/// Diagnostics
pub mod diagnostics;

/// OrCo Intermediate Representation lives here
pub mod ir;

/// Source and span
pub mod source;
pub use source::*;

/// Type inference structs and functions
pub mod type_inference;
pub use type_inference::TypeInference;

/// Interpreter for the IR
pub mod interpreter;
pub use interpreter::{Interpreter, Value};

/// Name, a segment of a [Path]
pub type Name = Span;

/// Path to a symbol
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Path(pub Vec<Name>);

impl Path {
    /// Create a new path
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new path with a single segment
    pub fn single(root: Name) -> Self {
        Self(vec![root])
    }

    /// Append a segment to the end of the path
    pub fn push(&mut self, segment: Name) {
        self.0.push(segment);
    }

    /// Same as [`Self::push`], but returns a new path instead of mutating this path
    pub fn extend(&self, segment: Name) -> Self {
        let mut path = self.clone();
        path.push(segment);
        path
    }
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, segment) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, "::")?;
            }
            write!(f, "{}", segment)?;
        }
        Ok(())
    }
}
