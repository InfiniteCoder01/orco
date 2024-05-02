use super::*;

/// Parse a function signature (assumes, that "fn" token is already consumed)
pub fn parse<R: ErrorReporter + ?Sized>(parser: &mut Parser<R>) -> ir::item::function::Signature {
    let start = parser.span().1.start;
    parser.expect_operator(Operator::LParen);
    let mut args = Vec::new();
    while !parser.match_operator(Operator::RParen) {
        let name = parser
            .expect_ident("argument name")
            .unwrap_or(parser.wrap_point("_".to_owned()));
        parser.expect_operator(Operator::Colon);
        let r#type = r#type::parse(parser);
        args.push((name, r#type));
        if !parser.match_operator(Operator::Comma) {
            parser.expect_operator(Operator::RParen);
            break;
        }
    }
    let args = parser.wrap_span(args, start);
    let return_type = if parser.match_operator(Operator::Arrow) {
        r#type::parse(parser)
    } else {
        parser.wrap_point(ir::Type::unit())
    };
    ir::item::function::Signature::new(args, return_type)
}

/// Parse a function signature with a name (assumes, that "fn" token is already consumed)
pub fn parse_named<R: ErrorReporter + ?Sized>(
    parser: &mut Parser<R>,
) -> Option<Named<ir::item::function::Signature>> {
    parser
        .expect_ident("function name")
        .map(|name| Named::new(name.inner, parse(parser)))
}
