use super::*;

/// Parse a function signature (assumes, that "fn" token is already consumed)
/// If parse_name is true, function name is expected
/// If register_args is true, registers arguments with parser's symbol mapper
pub fn parse<R: ErrorReporter + ?Sized>(
    parser: &mut Parser<R>,
    parse_name: bool,
) -> ir::symbol::function::Signature {
    let name = if parse_name {
        parser.expect_ident("function name")
    } else {
        None
    };
    let start = parser.span().1.start;
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
        );
        let declaration = parser.wrap_span(declaration, start);
        *declaration.id.lock().unwrap() = args.len() as _;
        args.push(std::sync::Arc::new(declaration));

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
    ir::symbol::function::Signature::new(name, args, return_type)
}
