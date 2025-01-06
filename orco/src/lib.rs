#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![feature(unsize)]

pub use orco_procmacro::*;

/// See [Expression]
pub mod expression;
pub use expression::Expression;

/// See [Type]
pub mod types;
pub use types::Type;

/// See [TypeInferenceContext]
pub mod type_inference;
pub use type_inference::TypeInferenceContext;

/// Symbol references are one of the key features of OrCo.
/// They allow symbols to be accessed from anywhere
pub mod symbol_box;
pub use symbol_box::{SymbolBox, SymbolRef};

/// `Cow<str>`
pub type CowStr<'a> = std::borrow::Cow<'a, str>;

/// `Arc<RwLock<T>>`
pub type ArcLock<T> = std::sync::Arc<std::sync::RwLock<T>>;
