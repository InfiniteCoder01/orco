use super::*;

/// Parse a binary expression with a set level of precedance
pub fn binary<R: ErrorReporter + ?Sized>(
    parser: &mut Parser<R>,
    level: usize,
) -> Option<Expression> {
    use ir::expression::BinaryOp;
    let operators = [
        vec![
            (Operator::EqualEqual, BinaryOp::Eq),
            (Operator::NotEqual, BinaryOp::Ne),
            (Operator::Lt, BinaryOp::Lt),
            (Operator::LtEq, BinaryOp::Le),
            (Operator::Gt, BinaryOp::Gt),
            (Operator::GtEq, BinaryOp::Ge),
        ],
        vec![
            (Operator::Plus, BinaryOp::Add),
            (Operator::Minus, BinaryOp::Sub),
        ],
        vec![
            (Operator::Star, BinaryOp::Mul),
            (Operator::Slash, BinaryOp::Div),
            (Operator::Percent, BinaryOp::Mod),
        ],
    ];

    if level >= operators.len() {
        return unary(parser);
    }

    let mut expression = binary(parser, level + 1)?;
    loop {
        let mut any = false;
        for &(op_token, op) in &operators[level] {
            if parser.match_operator(op_token) {
                let lhs = Box::new(expression);
                let rhs = Box::new(binary(parser, level + 1)?);
                let span = lhs.span().extend(&rhs.span());
                expression = Expression::BinaryExpression(Spanned::new(
                    ir::expression::BinaryExpression::new(lhs, op, rhs),
                    span,
                ));
                any = true;
            }
        }
        if !any {
            break;
        }
    }
    Some(expression)
}

/// Parse a unary expression
pub fn unary<R: ErrorReporter + ?Sized>(parser: &mut Parser<R>) -> Option<Expression> {
    use ir::expression::UnaryOp;
    let operators = [(Operator::Minus, UnaryOp::Neg)];
    for &(op_token, op) in &operators {
        let start = parser.span().1.start;
        if parser.match_operator(op_token) {
            let expr = Box::new(unary(parser)?);
            return Some(Expression::UnaryExpression(
                parser.wrap_span(ir::expression::UnaryExpression::new(op, expr), start),
            ));
        }
    }
    super::unit_expression(parser)
}
