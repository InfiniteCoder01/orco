use super::*;

/// Parsers for a function signature
pub mod signature;

/// Parse a function (assumes, that "fn" token is already consumed)
pub fn parse<R: ErrorReporter + ?Sized>(parser: &mut Parser<R>) -> ir::symbol::Function {
    parser.symbol_mapper.push_scope();
    let function = ir::symbol::Function::new(
        signature::parse(parser, true),
        expression::block::expect(parser),
    );
    parser.symbol_mapper.pop_scope();
    function
}

/// Parse a function with a name (assumes, that "fn" token is already consumed)
pub fn parse_named<R: ErrorReporter + ?Sized>(
    parser: &mut Parser<R>,
) -> Option<Named<ir::symbol::Function>> {
    parser
        .expect_ident("function name")
        .map(|name| Named::new(name, parse(parser)))
}
