use super::*;

/// Parsers for a function signature
pub mod signature;

/// Parse a function (assumes, that "fn" token is already consumed)
pub fn parse(parser: &mut Parser) -> ir::item::Function {
    let mut variable_mapper = orco::variable_mapper::VariableMapper::new();
    ir::item::Function::new(
        signature::parse(parser),
        expression::block::expect(parser, &mut variable_mapper),
    )
}

/// Parse a function with a name (assumes, that "fn" token is already consumed)
pub fn parse_named(parser: &mut Parser) -> Option<Named<ir::item::Function>> {
    parser
        .expect_ident("function name")
        .map(|name| Named::new(name.inner, parse(parser)))
}
