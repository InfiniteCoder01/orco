use super::*;

/// Parsers for a function signature
pub mod signature;

/// Parse a function (assumes, that "fn" token is already consumed)
pub fn parse<R: ErrorReporter + ?Sized>(parser: &mut Parser<R>) -> ir::symbol::Function {
    let mut variable_mapper = orco::symbol_mapper::SymbolMapper::new();
    ir::symbol::Function::new(
        signature::parse(parser, Some(&mut variable_mapper)),
        expression::block::expect(parser, &mut variable_mapper),
    )
}

/// Parse a function with a name (assumes, that "fn" token is already consumed)
pub fn parse_named<R: ErrorReporter + ?Sized>(
    parser: &mut Parser<R>,
) -> Option<Named<ir::symbol::Function>> {
    parser
        .expect_ident("function name")
        .map(|name| Named::new(name, parse(parser)))
}
