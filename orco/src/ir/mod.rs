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
    pub symbols: std::collections::HashMap<Name, Box<std::sync::Mutex<Symbol>>>,
}

impl Module {
    /// Infer and check types for the whole module
    pub fn infer_and_check_types(&self, type_inference: &mut TypeInference) {
        for symbol in self.symbols.values() {
            let mut symbol = symbol.lock().unwrap();
            symbol.value.infer_types(type_inference);
            symbol.value.finish_and_check_types(type_inference);
        }
    }

    /// Evaluate comptime symbols, has to be done before building
    pub fn evaluate_comptimes(&self, interpreter: &mut Interpreter) {
        for symbol in self.symbols.values() {
            let mut symbol = symbol.lock().unwrap();
            if symbol.evaluated.is_none() {
                symbol.evaluated = Some(interpreter.evaluate(&symbol.value));
            }
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
                indent::indent_all_by(4, format!("{}", symbol.lock().unwrap()))
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
