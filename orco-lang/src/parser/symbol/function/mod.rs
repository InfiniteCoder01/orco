use super::*;

/// Parsers for a function signature
pub mod signature;

/// Parse a function (assumes, that "fn" token is already consumed)
/// If parse_name is true, function name is expected
pub fn parse<R: ErrorReporter + ?Sized>(
    parser: &mut Parser<R>,
) -> ir::symbol::Function {
    let function = ir::symbol::Function::new(
        signature::parse(parser),
        expression::block::expect(parser),
    );
    function
}
