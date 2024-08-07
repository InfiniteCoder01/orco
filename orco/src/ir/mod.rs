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
#[derive(Debug, Default)]
pub struct Module {
    /// Module content
    pub symbols: std::collections::HashMap<Name, std::pin::Pin<Box<std::sync::RwLock<Symbol>>>>,
}

impl Module {
    /// Infer and check types for the whole module
    pub fn infer_and_check_types(&self, type_inference: &mut TypeInference) {
        for symbol in self.symbols.values() {
            symbol::ensure_evaluated(symbol, type_inference);
        }
    }
}

impl Clone for Module {
    fn clone(&self) -> Self {
        Self {
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

#[macro_export]
/// Create a new metadata trait
macro_rules! declare_metadata {
    (
        $(
            $(#[$meta:meta])*
            trait $trait_name:ident {
                $(
                    $(#[$fn_meta:meta])*
                    fn $fn_name:ident$(<$($lt:lifetime),*>)?($($args:tt)*) $(-> $ret:ty)? $fn_body:block
                )*
            }
        )*
    ) => {
        $(
            $(#[$meta])*
            pub trait $trait_name: Downcast + DynClone + Send + Sync {
                $(
                    $(#[$fn_meta])*
                    fn $fn_name $(<$($lt),*>)? ($($args)*) $(-> $ret)? $fn_body
                )*
            }

            impl_downcast!($trait_name);
            clone_trait_object!($trait_name);
            impl $trait_name for () {}
        )*
    };
}
