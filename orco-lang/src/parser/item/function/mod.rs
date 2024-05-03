use super::*;

/// Parsers for a function signature
pub mod signature;

/// Parse a function (assumes, that "fn" token is already consumed)
pub fn parse<R: ErrorReporter + ?Sized>(parser: &mut Parser<R>) -> ir::item::Function {
    let mut variable_mapper = orco::variable_mapper::VariableMapper::new();
    ir::item::Function::new(
        signature::parse(parser, Some(&mut variable_mapper)),
        expression::block::expect(parser, &mut variable_mapper),
    )
}

/// Parse a function with a name (assumes, that "fn" token is already consumed)
pub fn parse_named<R: ErrorReporter + ?Sized>(
    parser: &mut Parser<R>,
) -> Option<Named<ir::item::Function>> {
    parser
        .expect_ident("function name")
        .map(|name| Named::new(name.inner, parse(parser)))
}
