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

/// A segment of a [Path]
pub type PathSegment = Span;

/// Path to a symbol
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Path(pub Vec<PathSegment>);

impl Path {
    /// Create a new path
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new path with a single segment
    pub fn single(root: PathSegment) -> Self {
        Self(vec![root])
    }

    /// Append a segment to the end of the path
    pub fn push(&mut self, segment: PathSegment) {
        self.0.push(segment);
    }

    /// Same as [`Self::push`], but returns a new path instead of mutating this path
    pub fn extend(&self, segment: PathSegment) -> Self {
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
