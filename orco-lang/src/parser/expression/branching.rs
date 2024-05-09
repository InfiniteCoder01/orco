use super::*;

/// Expect an if expression, assuming that 'if' keyword was already consumed
pub fn expect_if<R: ErrorReporter + ?Sized>(
    parser: &mut Parser<R>,
    variable_mapper: &mut SymbolMapper,
    start: usize,
) -> Option<Expression> {
    let condition = Box::new(expect(parser, variable_mapper));
    let then_branch = Box::new(expect(parser, variable_mapper));
    let else_branch = if parser.match_keyword("else") {
        Some(Box::new(expect(parser, variable_mapper)))
    } else {
        None
    };
    Some(Expression::If(parser.wrap_span(
        ir::expression::IfExpression::new(condition, then_branch, else_branch),
        start,
    )))
}
