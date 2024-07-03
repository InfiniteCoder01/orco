use crate::*;
use derivative::Derivative;
use downcast_rs::{impl_downcast, Downcast};
use dyn_clone::{clone_trait_object, DynClone};

/// Of course we are statically-typed
pub mod types;
pub use types::Type;

/// All kinds of symbols
pub mod symbol;
pub use symbol::Symbol;

/// All kinds of expressions (statements are expressions as well)
pub mod expression;
pub use expression::Expression;

/// A module (namespace), can be a file, some small section of it or the whole project
#[derive(Debug, Default)]
pub struct Module {
    /// Module content
    pub symbols: Vec<Symbol>,
    /// Symbol map, can be used to resolve symbols
    pub symbol_map: std::collections::HashMap<PathSegment, Vec<SymbolReference>>,
}

impl Module {
    /// Register all symbols in the module
    pub fn register(&mut self) {
        for symbol in &self.symbols {
            match symbol {
                Symbol::Function(function) => {
                    self.symbol_map
                        .entry(function.signature.name.clone())
                        .or_default()
                        .push(crate::SymbolReference::Function(function.clone()));
                }

                Symbol::ExternalFunction(function) => {
                    self.symbol_map
                        .entry(function.name.clone())
                        .or_default()
                        .push(crate::SymbolReference::ExternFunction(function.clone()));
                }
            }
        }
    }

    /// Infer types for the whole module
    pub fn infer_and_check_types(
        &self,
        reporter: &mut dyn crate::diagnostics::ErrorReporter,
        root_module: &Module,
        current_path: &Path,
    ) {
        for symbol in &self.symbols {
            if let Symbol::Function(function) = symbol {
                function.infer_and_check_types(reporter, root_module, self, current_path);
            }
        }
    }
}

impl std::fmt::Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for symbol in &self.symbols {
            writeln!(f, "{}\n", symbol)?;
        }
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
                    fn $fn_name:ident ($($args:tt)*) $(-> $ret:ty)? $fn_body:block
                )*

                $(
                    Diagnostics:
                    $(
                        $(#[$diagnostic_meta:meta])*
                        $diagnostic_handler_name:ident ($diagnostic_name:ident)
                    )*
                )?
            }
        )*
    ) => {
        $(
            $(#[$meta])*
            pub trait $trait_name: Downcast + DynClone + Send + Sync {
                $(
                    $(#[$fn_meta])*
                    fn $fn_name ($($args)*) $(-> $ret)? $fn_body
                )*

                $(
                    $(
                        $(#[$diagnostic_meta])*
                        fn $diagnostic_handler_name (&self, type_inference: &mut TypeInference, diagnostic: $diagnostic_name) {
                            type_inference.reporter.report(diagnostic.into());
                        }
                    )*
                )?
            }

            impl_downcast!($trait_name);
            clone_trait_object!($trait_name);
            impl $trait_name for () {}
        )*
    };
}
