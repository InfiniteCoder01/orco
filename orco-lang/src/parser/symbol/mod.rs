use super::*;

/// Parsers for different parts of a function
pub mod function;

/// Parse a symbol (it there is one, otherwise returns None and does nothing)
pub fn parse<R: ErrorReporter + ?Sized>(parser: &mut Parser<R>) -> Option<ir::Symbol> {
    if parser.match_keyword("fn") {
        let function = function::parse(parser);
        Some(ir::Symbol::Function(std::sync::Arc::new(function)))
    } else if parser.match_keyword("extern") {
        if parser.match_keyword("fn") {
            let function = function::signature::parse(parser);
            parser.expect_operator(Operator::Semicolon);
            Some(ir::Symbol::ExternalFunction(std::sync::Arc::new(function)))
        } else {
            parser.expected_error("extern symbol");
            None
        }
    } else {
        None
    }
}
