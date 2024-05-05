use crate::lexer::*;
use orco::diagnostics::*;
use orco::ir;

/// Parsers for expressions
pub mod expression;
/// Parsers for symbols (e.g. Function or Extern)
pub mod symbol;
/// Parsers for types
pub mod r#type;

/// Parse the whole file
pub fn parse<R: ErrorReporter + ?Sized>(parser: &mut Parser<R>) -> ir::Module {
    let mut module = ir::Module::default();
    while !parser.eof() {
        if let Some(symbol) = symbol::parse(parser) {
            module.symbols.insert(symbol.name, symbol.value);
        } else {
            parser.expected_error("a symbol");
            parser.next();
        }
    }
    module
}

/// A named object (for example, if you parse a Function, you'll get Named<Function>, because the
/// function itself doen't store a name)
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Named<T> {
    /// Name
    pub name: Span,
    /// Value
    pub value: T,
}

impl<T> Named<T> {
    /// Create a new named object
    pub fn new(name: Span, value: T) -> Self {
        Self { name, value }
    }

    /// Map the value, preserving the name
    pub fn map<U>(self, mapper: impl Fn(T) -> U) -> Named<U> {
        Named {
            name: self.name,
            value: mapper(self.value),
        }
    }
}

impl<T> std::ops::Deref for Named<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> std::ops::DerefMut for Named<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
