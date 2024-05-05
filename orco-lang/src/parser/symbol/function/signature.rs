use super::*;

/// Parse a function signature (assumes, that "fn" token is already consumed)
pub fn parse<R: ErrorReporter + ?Sized>(
    parser: &mut Parser<R>,
    mut variable_mapper: Option<&mut orco::variable_mapper::VariableMapper>,
) -> ir::symbol::function::Signature {
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

        let declaration = ir::expression::variable_declaration::VariableDeclaration {
            name,
            id: args.len() as _,
            mutable: parser.wrap_point(false),
            r#type,
            value: None,
        };
        let declaration = parser.wrap_span(declaration, start);
        let declaration = if let Some(variable_mapper) = &mut variable_mapper {
            variable_mapper.declare_variable(declaration)
        } else {
            std::sync::Arc::new(declaration.map(std::sync::Mutex::new))
        };
        args.push(declaration);

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
    ir::symbol::function::Signature::new(args, return_type)
}

/// Parse a function signature with a name (assumes, that "fn" token is already consumed)
pub fn parse_named<R: ErrorReporter + ?Sized>(
    parser: &mut Parser<R>,
) -> Option<Named<ir::symbol::function::Signature>> {
    parser
        .expect_ident("function name")
        .map(|name| Named::new(name, parse(parser, None)))
}
