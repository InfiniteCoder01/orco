use super::*;

/// Parse a block
pub fn parse<R: ErrorReporter + ?Sized>(parser: &mut Parser<R>) -> Option<ir::expression::Block> {
    let start = parser.span().1.start;
    if parser.match_operator(Operator::LBrace) {
        let mut expressions = Vec::new();
        let tail_expression = 'a: {
            while !parser.match_operator(Operator::RBrace) {
                match expression::expect(parser) {
                    ir::Expression::Error(_) => {
                        parser.next();
                    }
                    expression => {
                        if parser.match_operator(Operator::RBrace) {
                            break 'a Some(Box::new(expression));
                        }
                        if !expression.is_block() {
                            parser.expect_operator(Operator::Semicolon);
                        } else {
                            parser.match_operator(Operator::Semicolon);
                        }
                        expressions.push(expression);
                    }
                }
            }
            None
        };
        Some(ir::expression::Block::new(
            expressions,
            tail_expression,
            parser.span_from(start),
            false,
            (),
        ))
    } else {
        None
    }
}

/// Expect a block
pub fn expect<R: ErrorReporter + ?Sized>(parser: &mut Parser<R>) -> ir::expression::Block {
    if let Some(block) = parse(parser) {
        block
    } else {
        parser.expected_error("a block");
        ir::expression::Block::new(Vec::new(), None, parser.point_span(), false, ())
    }
}
