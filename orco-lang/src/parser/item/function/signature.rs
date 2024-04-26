use super::*;

/// Parse a function signature (assumes, that "fn" token is already consumed)
pub fn parse(parser: &mut Parser) -> ir::item::function::Signature {
    parser.expect_operator(Operator::LParen);
    let mut args = Vec::new();
    while !parser.match_opertor(Operator::RParen) {
        let name = parser
            .expect_ident("argument name")
            .unwrap_or("_".to_owned());
        parser.expect_operator(Operator::Colon);
        let r#type = r#type::parse(parser);
        args.push((name, r#type));
        if !parser.match_opertor(Operator::Comma) {
            parser.expect_operator(Operator::RParen);
            break;
        }
    }
    let return_type = if parser.match_opertor(Operator::Arrow) {
        r#type::parse(parser)
    } else {
        parser.wrap_point(ir::Type::unit())
    };
    ir::item::function::Signature::new(args, return_type)
}

/// Parse a function signature with a name (assumes, that "fn" token is already consumed)
pub fn parse_named(parser: &mut Parser) -> Option<Named<ir::item::function::Signature>> {
    parser
        .expect_ident("function name")
        .map(|name| Named::new(name, parse(parser)))
}
