use crate::lexer::*;
use orco::diagnostics::*;
use orco::ir;

/// Parsers for expressions
pub mod expression;
/// Parsers for types
pub mod r#type;

/// Parse a symbol
pub fn parse_symbol<R: ErrorReporter + ?Sized>(parser: &mut Parser<R>) -> Option<ir::Symbol> {
    if !parser.match_keyword("comptime") {
        return None;
    }
    let name = parser.expect_ident("symbol name")?;
    parser.expect_operator(Operator::Equal);
    let value = expression::expect(parser);
    Some(ir::Symbol::new(name, value))
}

/// Parse the whole file
pub fn parse<R: ErrorReporter + ?Sized>(parser: &mut Parser<R>) -> ir::Module {
    let mut module = ir::Module::default();
    while !parser.eof() {
        if let Some(symbol) = parse_symbol(parser) {
            parser.expect_operator(Operator::Semicolon);
            module
                .symbols
                .insert(symbol.name.clone(), Box::new(std::sync::RwLock::new(symbol)));
        } else {
            parser.expected_error("a symbol");
            parser.next();
        }
    }
    module
}
