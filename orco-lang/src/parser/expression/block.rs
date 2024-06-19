use super::*;

/// Parse a block
pub fn parse<R: ErrorReporter + ?Sized>(
    parser: &mut Parser<R>,
) -> Option<Spanned<ir::expression::Block>> {
    let start = parser.span().1.start;
    if parser.match_operator(Operator::LBrace) {
        let mut block = ir::expression::Block::default();
        while !parser.match_operator(Operator::RBrace) {
            match expression::expect(parser) {
                ir::Expression::Error(_) => {
                    parser.next();
                }
                expression => {
                    if parser.match_operator(Operator::RBrace) {
                        block.tail_expression = Some(Box::new(expression));
                        break;
                    }
                    if !expression.is_block() {
                        parser.expect_operator(Operator::Semicolon);
                    } else {
                        parser.match_operator(Operator::Semicolon);
                    }
                    block.expressions.push(expression);
                }
            }
        }
        Some(parser.wrap_span(block, start))
    } else {
        None
    }
}

/// Expect a block
pub fn expect<R: ErrorReporter + ?Sized>(parser: &mut Parser<R>) -> Spanned<ir::expression::Block> {
    if let Some(block) = parse(parser) {
        block
    } else {
        parser.expected_error("a block");
        parser.wrap_point(ir::expression::Block::default())
    }
}
