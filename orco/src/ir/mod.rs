use crate::*;
use derivative::Derivative;
use downcast_rs::{impl_downcast, Downcast};
use dyn_clone::{clone_trait_object, DynClone};

/// Types
pub mod types;
pub use types::Type;

/// Symbol - a value that can be exported or used internally, evaluated at compile time
pub mod symbol;
pub use symbol::Symbol;

/// Expressions (and statements)
pub mod expression;
pub use expression::Expression;

/// A module (namespace). Can be a file, some small section of it or the whole codebase
#[derive(Derivative)]
#[derivative(Debug, Default)]
pub struct Module {
    /// Parent
    pub parent: Option<expression::symbol_reference::InternalPointer<Module>>,
    /// Module content
    pub symbols: std::collections::HashMap<Name, std::pin::Pin<Box<std::sync::RwLock<Symbol>>>>,
    /// Metadata
    #[derivative(Debug = "ignore", Default(value = "Box::new(())"))]
    pub metadata: Box<dyn ModuleMetadata>,
}

impl Module {
    /// Create a new module
    pub fn new(metadata: impl ModuleMetadata + 'static) -> Box<Self> {
        Box::new(Self {
            parent: None,
            symbols: std::collections::HashMap::new(),
            metadata: Box::new(metadata),
        })
    }

    /// Infer and check types for the whole module
    pub fn infer_and_check_types(self: std::pin::Pin<&Self>, type_inference: &mut TypeInference) {
        for symbol in self.symbols.values() {
            symbol::ensure_evaluated(symbol, type_inference);
        }
    }
}

impl Clone for Module {
    fn clone(&self) -> Self {
        if self.parent.is_some() {
            panic!("Can't clone inferred module (has parent)!");
        }
        Self {
            parent: None,
            symbols: self
                .symbols
                .iter()
                .map(|(name, symbol)| {
                    (
                        name.clone(),
                        Box::pin(std::sync::RwLock::new(symbol.try_read().unwrap().clone())),
                    )
                })
                .collect(),
            metadata: self.metadata.clone(),
        }
    }
}

impl std::fmt::Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "module {{")?;
        for symbol in self.symbols.values() {
            writeln!(
                f,
                "{}",
                indent::indent_all_by(4, format!("{}", symbol.try_read().unwrap()))
            )?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

/// Frontend metadata for a module
pub trait ModuleMetadata: Metadata {}
impl_metadata!(ModuleMetadata);

/// Any metadata implements this trait
pub trait Metadata: Downcast + DynClone + Send + Sync {}
impl Metadata for () {}

#[macro_export]
/// Create a new metadata trait
macro_rules! impl_metadata {
    ($trait:ident) => {
        impl_downcast!($trait);
        clone_trait_object!($trait);
        impl $trait for () {}
    };
}
