/// Of course we are statically-typed
pub mod types;
pub use types::Type;

/// All kinds of items
pub mod item;
pub use item::Item;

/// All kinds of expressions (statements are expressions)
pub mod expression;
pub use expression::Expression;

/// A module, can be one file or the whole project
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Module {
    /// Module content
    pub items: std::collections::HashMap<String, Item>,
}
