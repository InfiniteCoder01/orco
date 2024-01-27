use super::*;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
/// An intermediate representation of a module/namespace
pub struct Module {
    items: HashMap<Symbol, Item>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// An intermediate representation of an item
pub struct Item {}

// impl std::fmt::Debug for Module {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let mut module = f.debug_struct("Module");
//         for (&name, item) in &self.items {
//             module.field(resolve_interned(name), item);
//         }
//         module.finish()
//     }
// }

// impl std::fmt::Debug for Item {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         Ok(())
//     }
// }
