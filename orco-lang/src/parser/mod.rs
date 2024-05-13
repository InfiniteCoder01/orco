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
            module.symbols.push(symbol);
        } else {
            parser.expected_error("a symbol");
            parser.next();
        }
    }
    module
}
