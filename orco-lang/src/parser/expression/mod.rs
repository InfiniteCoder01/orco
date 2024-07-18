use super::*;
use orco::ir::Expression;

/// Parsers for operator-oriented expressions (binary, unary, assignment, etc.)
pub mod operator;

/// Parsers for a block expression
pub mod block;

/// Parsers for branching constructs
pub mod branching;

/// Expect an expression, error if there is no
pub fn expect<R: ErrorReporter + ?Sized>(parser: &mut Parser<R>) -> Expression {
    if let Some(expression) = parse(parser) {
        expression
    } else {
        parser.expected_error("expression");
        Expression::Error(parser.span())
    }
}

/// Parse an expression
pub fn parse<R: ErrorReporter + ?Sized>(parser: &mut Parser<R>) -> Option<Expression> {
    let start = parser.span().1.start;
    let expression = if parser.match_keyword("return") {
        let value = expect(parser);
        Some(Expression::Return(parser.wrap_span(
            ir::expression::ReturnExpression(Box::new(value), Box::new(())),
            start,
        )))
    } else if parser.match_keyword("let") {
        let mutable = {
            let span = parser.span();
            Spanned {
                inner: parser.match_keyword("mut"),
                span,
            }
        };
        let name = parser.expect_ident("variable name")?;
        let r#type = if parser.match_operator(Operator::Colon) {
            r#type::parse(parser)
        } else {
            parser.wrap_point(ir::Type::Wildcard)
        };
        let value = if parser.match_operator(Operator::Equal) {
            Some(expect(parser))
        } else {
            None
        };
        Some(Expression::VariableDeclaration(std::sync::Arc::new(
            parser.wrap_span(
                ir::expression::VariableDeclaration::new(name, mutable, r#type, value),
                start,
            ),
        )))
    } else {
        operator::binary(parser, 0)
    };
    if let Some(expression) = expression {
        if parser.match_operator(Operator::Equal) {
            let target = Box::new(expression);
            let value = Box::new(expect(parser));
            Some(Expression::Assignment(parser.wrap_span(
                ir::expression::AssignmentExpression::new(target, value, Box::new(())),
                start,
            )))
        } else {
            Some(expression)
        }
    } else {
        expression
    }
}

/// Parse a unit expression
pub fn unit_expression<R: ErrorReporter + ?Sized>(parser: &mut Parser<R>) -> Option<Expression> {
    let start = parser.span().1.start;
    let expr = if parser.match_operator(Operator::LParen) {
        let expr = expect(parser);
        parser.expect_operator(Operator::RParen);
        expr
    } else if let Some(span) = parser.match_error() {
        Expression::Error(span)
    } else if let Some(constant) = parser.match_constant() {
        Expression::Constant(constant)
    } else if let Some(block) = block::parse(parser) {
        Expression::Block(block)
    } else if parser.match_keyword("if") {
        branching::expect_if(parser, start)
    } else if let Some(name) = parser.match_ident() {
        Expression::Symbol(
            parser.wrap_span(
                orco::SymbolReference::Undeclared(orco::Path::single(name)),
                start,
            ),
            Box::new(()),
        )
    } else {
        return None;
    };

    let args_start = parser.span().1.start;
    Some(if parser.match_operator(Operator::LParen) {
        let mut args = Vec::new();
        while let Some(expression) = parse(parser) {
            args.push(expression);
            if !parser.match_operator(Operator::Comma) {
                break;
            }
        }
        parser.expect_operator(Operator::RParen);
        let args = parser.wrap_span(args, args_start);
        Expression::Call(parser.wrap_span(
            ir::expression::CallExpression::new(expr, args, Box::new(())),
            start,
        ))
    } else {
        expr
    })
}
