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
    /// Infer types in the whole module
    pub fn infer_types(&mut self) {
        for item in self.items.values_mut() {
            if let Item::Function(function) = item {
                function.infer_types();
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
