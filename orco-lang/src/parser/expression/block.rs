use super::*;

/// Parse a block
pub fn parse(parser: &mut Parser) -> Option<ir::expression::Block> {
    if parser.match_opertor(Operator::LBrace) {
        let mut block = ir::expression::Block::default();
        while !parser.match_opertor(Operator::RBrace) {
            match expression::expect(parser) {
                ir::Expression::Error => {
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
        Some(block)
    } else {
        None
    }
}

/// Expect a block
pub fn expect(parser: &mut Parser) -> ir::expression::Block {
    if let Some(block) = parse(parser) {
        block
    } else {
        parser.expected_error("a block");
        ir::expression::Block::default()
    }
}
