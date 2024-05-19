use super::*;

/// Parsers for different parts of a function
pub mod function;

/// Parse a symbol (it there is one, otherwise returns None and does nothing)
pub fn parse<R: ErrorReporter + ?Sized>(parser: &mut Parser<R>) -> Option<ir::Symbol> {
    let start = parser.span().1.start;
    if parser.match_keyword("fn") {
        let function = function::parse(parser);
        let symbol = std::sync::Arc::new(parser.wrap_span(function, start));
        Some(ir::Symbol::Function(symbol))
    } else if parser.match_keyword("extern") {
        if parser.match_keyword("fn") {
            let function = function::signature::parse(parser);
            let symbol = std::sync::Arc::new(parser.wrap_span(function, start));
            parser.expect_operator(Operator::Semicolon);
            Some(ir::Symbol::ExternalFunction(symbol))
        } else {
            parser.expected_error("extern symbol");
            None
        }
    } else {
        None
    }
}
