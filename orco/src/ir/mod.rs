use crate::source::*;

/// Of course we are statically-typed
pub mod types;
pub use types::Type;

/// All kinds of symbols
pub mod symbol;
pub use symbol::Symbol;

/// All kinds of expressions (statements are expressions as well)
pub mod expression;
pub use expression::Expression;

/// A module, can be one file or the whole project
#[derive(Debug, Default)]
pub struct Module {
    /// Module content
    pub symbols: Vec<Symbol>,
}

impl Module {
    /// Infer types for the whole module
    pub fn infer_and_check_types(&self, reporter: &mut dyn crate::diagnostics::ErrorReporter) {
        let mut global_scope = crate::type_inference::Scope::new();
        for symbol in &self.symbols {
            match symbol {
                Symbol::Function(function) => {
                    if let Some(name) = &function.signature.name {
                        global_scope.insert(
                            name.clone(),
                            crate::SymbolReference::Function(function.clone()),
                        );
                    } else {
                        panic!("Declaring unnamed external function in global scope!")
                    }
                }

                Symbol::ExternalFunction(function) => {
                    if let Some(name) = &function.name {
                        global_scope.insert(
                            name.clone(),
                            crate::SymbolReference::ExternFunction(function.clone()),
                        );
                    } else {
                        panic!("Declaring unnamed external function in global scope!")
                    }
                }
            }
        }

        for symbol in &self.symbols {
            if let Symbol::Function(function) = symbol {
                function.infer_and_check_types(reporter, &mut global_scope);
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
