use super::*;

/// Parse a block
pub fn parse<R: ErrorReporter + ?Sized>(parser: &mut Parser<R>) -> Option<ir::expression::Block> {
    let start = parser.span().1.start;
    if parser.match_operator(Operator::LBrace) {
        let mut expressions = Vec::new();
        let mut comptimes = Vec::new();
        let tail_expression = 'a: {
            while !parser.match_operator(Operator::RBrace) {
                if let Some(symbol) = parse_symbol(parser) {
                    comptimes.push(Box::pin(std::sync::RwLock::new(symbol)));
                    continue;
                }
                match expression::expect(parser) {
                    ir::Expression::Error(_) => {
                        if parser.next().is_none() {
                            parser.expect_operator(Operator::RBrace);
                            break;
                        }
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
            comptimes,
            expressions,
            tail_expression,
            false,
            Some(parser.span_from(start)),
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
        ir::expression::Block {
            span: Some(parser.point_span()),
            ..Default::default()
        }
    }
}
