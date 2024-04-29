use crate::lexer::*;
use orco::diagnostics::Spanned;
use orco::ir;

/// Parsers for expressions
pub mod expression;
/// Parsers for items (e.g. Function or Extern)
pub mod item;
/// Parsers for types
pub mod r#type;

/// Parse the whole file
pub fn parse(parser: &mut Parser) -> ir::Module {
    let mut module = ir::Module::default();
    while !parser.eof() {
        if let Some(item) = item::parse(parser) {
            module.items.insert(item.name, item.value);
        } else {
            parser.expected_error("an item");
            parser.next();
        }
    }
    module
}

/// A named item (for example, if you parse a Function, you'll get Named<Function>, because the
/// function itself doen't store a name)
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Named<T> {
    /// Name
    pub name: String,
    /// Value
    pub value: T,
}

impl<T> Named<T> {
    /// Create a new named item
    pub fn new(name: String, item: T) -> Self {
        Self { name, value: item }
    }

    /// Map the value, preserving the name
    pub fn map<U>(self, mapper: impl Fn(T) -> U) -> Named<U> {
        Named {
            name: self.name,
            value: mapper(self.value),
        }
    }
}
