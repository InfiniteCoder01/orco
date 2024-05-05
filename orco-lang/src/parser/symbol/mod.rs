use super::*;

/// Parsers for different parts of a function
pub mod function;

/// Parse a symbol (it there is one, otherwise returns None and does nothing)
pub fn parse<R: ErrorReporter + ?Sized>(parser: &mut Parser<R>) -> Option<Named<ir::Symbol>> {
    if parser.match_keyword("fn") {
        function::parse_named(parser).map(|symbol| symbol.map(ir::Symbol::Function))
    } else if parser.match_keyword("extern") {
        if parser.match_keyword("fn") {
            let symbol = function::signature::parse_named(parser)
                .map(|symbol| symbol.map(ir::Symbol::ExternalFunction));
            parser.expect_operator(Operator::Semicolon);
            symbol
        } else {
            parser.expected_error("extern symbol or block");
            None
        }
    } else {
        None
    }
}
