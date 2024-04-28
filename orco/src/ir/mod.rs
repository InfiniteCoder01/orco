use crate::diagnostics::Spanned;

/// Of course we are statically-typed
pub mod types;
pub use types::Type;

/// All kinds of items
pub mod item;
pub use item::Item;

/// All kinds of expressions (statements are expressions as well)
pub mod expression;
pub use expression::Expression;

/// A module, can be one file or the whole project
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Module {
    /// Module content
    pub items: std::collections::HashMap<String, Item>,
}

impl Module {
    /// Infer types for the whole module
    pub fn infer_and_check_types(&self, root: &Module, reporter: &mut dyn crate::diagnostics::ErrorReporter) {
        for item in self.items.values() {
            if let Item::Function(function) = item {
                function.infer_and_check_types(root, reporter);
            }
        }
    }
}

impl std::fmt::Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (name, item) in &self.items {
            item.format(f, Some(name))?;
            writeln!(f, "\n")?;
        }
        Ok(())
    }
}
