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
    pub symbols: std::collections::HashMap<Span, Symbol>,
}

impl Module {
    /// Infer types for the whole module
    pub fn infer_and_check_types(
        &self,
        root: &Module,
        reporter: &mut dyn crate::diagnostics::ErrorReporter,
    ) {
        for symbol in self.symbols.values() {
            if let Symbol::Function(function) = symbol {
                function.infer_and_check_types(root, reporter);
            }
        }
    }
}

impl std::fmt::Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (name, symbol) in &self.symbols {
            symbol.format(f, Some(name))?;
            writeln!(f, "\n")?;
        }
        Ok(())
    }
}
