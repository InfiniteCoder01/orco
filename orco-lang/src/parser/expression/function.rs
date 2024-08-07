use super::*;

/// Parse function signature
pub fn parse_signature<R: ErrorReporter + ?Sized>(
    parser: &mut Parser<R>,
) -> ir::expression::function::Signature {
    let args_start = parser.span().1.start;
    parser.expect_operator(Operator::LParen);
    let mut args = Vec::new();
    while !parser.match_operator(Operator::RParen) {
        let start = parser.span().1.start;
        let name = parser
            .expect_ident("argument name")
            .unwrap_or(parser.span());
        parser.expect_operator(Operator::Colon);
        let r#type = r#type::parse(parser);

        let declaration = ir::expression::variable_declaration::VariableDeclaration::new(
            name,
            parser.wrap_point(false),
            r#type,
            None,
            parser.span_from(start),
            (),
        );
        args.push(Box::pin(declaration));

        if !parser.match_operator(Operator::Comma) {
            parser.expect_operator(Operator::RParen);
            break;
        }
    }
    let args = parser.wrap_span(args, args_start);
    let return_type = if parser.match_operator(Operator::Arrow) {
        r#type::parse(parser)
    } else {
        parser.wrap_point(ir::Type::unit())
    };
    ir::expression::function::Signature::new(args, return_type)
}
