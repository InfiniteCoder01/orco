//! OrCo is the base crate for OrCo compiler toolchain.
//! It's used in parser and backend crates as a glue.
#![warn(missing_docs)]

/// Intermediate Representation lives here
pub mod ir;

/// Type inference information for a function
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeInferenceInfo<'a> {
    /// Root module
    pub root: &'a ir::Module,
    /// Return type of a function
    pub return_type: &'a ir::Type,
}

impl TypeInferenceInfo<'_> {
    /// Get function signature
    pub fn signature(&self, name: &str) -> Option<&ir::item::function::Signature> {
        self.root
            .items
            .get(name)
            .and_then(|item| item.function_signature())
    }
}
