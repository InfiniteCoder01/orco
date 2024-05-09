use super::*;

/// Parse a block
pub fn parse<R: ErrorReporter + ?Sized>(
    parser: &mut Parser<R>,
    variable_mapper: &mut SymbolMapper,
) -> Option<Spanned<ir::expression::Block>> {
    let start = parser.span().1.start;
    if parser.match_operator(Operator::LBrace) {
        variable_mapper.push_scope();
        let mut block = ir::expression::Block::default();
        while !parser.match_operator(Operator::RBrace) {
            match expression::expect(parser, variable_mapper) {
                ir::Expression::Error(_) => {
                    parser.next();
                }
                expression => {
                    if !expression.is_block() {
                        parser.expect_operator(Operator::Semicolon);
                    }
                    block.expressions.push(expression);
                }
            }
        }
        variable_mapper.pop_scope();
        Some(parser.wrap_span(block, start))
    } else {
        None
    }
}

/// Expect a block
pub fn expect<R: ErrorReporter + ?Sized>(
    parser: &mut Parser<R>,
    variable_mapper: &mut SymbolMapper,
) -> Spanned<ir::expression::Block> {
    if let Some(block) = parse(parser, variable_mapper) {
        block
    } else {
        parser.expected_error("a block");
        parser.wrap_point(ir::expression::Block::default())
    }
}
