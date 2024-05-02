use super::*;

/// Parsers for different parts of a function
pub mod function;

/// Parse an item (it there is an item, otherwise returns None and does nothing)
pub fn parse<R: ErrorReporter + ?Sized>(parser: &mut Parser<R>) -> Option<Named<ir::Item>> {
    if parser.match_keyword("fn") {
        function::parse_named(parser).map(|item| item.map(ir::Item::Function))
    } else if parser.match_keyword("extern") {
        if parser.match_keyword("fn") {
            let item = function::signature::parse_named(parser)
                .map(|item| item.map(ir::Item::ExternalFunction));
            parser.expect_operator(Operator::Semicolon);
            item
        } else {
            parser.expected_error("extern item or block");
            None
        }
    } else {
        None
    }
}
