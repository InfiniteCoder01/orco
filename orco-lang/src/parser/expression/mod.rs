use super::*;
use orco::ir::Expression;
use orco::symbol_mapper::SymbolMapper;

/// Parsers for operator-oriented expressions (binary, unary, assignment, etc.)
pub mod operator;

/// Parsers for a block expression
pub mod block;

/// Parsers for branching constructs
pub mod branching;

/// Expect an expression, error if there is no
pub fn expect<R: ErrorReporter + ?Sized>(
    parser: &mut Parser<R>,
    variable_mapper: &mut SymbolMapper,
) -> Expression {
    if let Some(expression) = parse(parser, variable_mapper) {
        expression
    } else {
        parser.expected_error("expression");
        Expression::Error(parser.span())
    }
}

/// Parse an expression
pub fn parse<R: ErrorReporter + ?Sized>(
    parser: &mut Parser<R>,
    variable_mapper: &mut SymbolMapper,
) -> Option<Expression> {
    let start = parser.span().1.start;
    let expression = if parser.match_keyword("return") {
        let value = expect(parser, variable_mapper);
        Some(Expression::Return(parser.wrap_span(Box::new(value), start)))
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
            Some(Box::new(expect(parser, variable_mapper)))
        } else {
            None
        };
        Some(Expression::VariableDeclaration(
            variable_mapper.declare_variable(parser.wrap_span(
                ir::expression::VariableDeclaration {
                    name,
                    id: 0,
                    mutable,
                    r#type,
                    value,
                },
                start,
            )),
        ))
    } else {
        operator::binary(parser, variable_mapper, 0)
    };
    if let Some(expression) = expression {
        if parser.match_operator(Operator::Equal) {
            let target = Box::new(expression);
            let value = Box::new(expect(parser, variable_mapper));
            Some(Expression::Assignment(parser.wrap_span(
                ir::expression::AssignmentExpression::new(target, value),
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
pub fn unit_expression<R: ErrorReporter + ?Sized>(
    parser: &mut Parser<R>,
    variable_mapper: &mut SymbolMapper,
) -> Option<Expression> {
    let start = parser.span().1.start;
    if let Some(span) = parser.match_error() {
        Some(Expression::Error(span))
    } else if let Some(constant) = parser.match_constant() {
        Some(Expression::Constant(constant))
    } else if let Some(block) = block::parse(parser, variable_mapper) {
        Some(Expression::Block(block))
    } else if parser.match_keyword("if") {
        branching::expect_if(parser, variable_mapper, start)
    } else if let Some(name) = parser.match_ident() {
        let start = parser.span().1.start;
        if parser.match_operator(Operator::LParen) {
            let mut args = Vec::new();
            while let Some(expression) = parse(parser, variable_mapper) {
                args.push(expression);
                if !parser.match_operator(Operator::Comma) {
                    break;
                }
            }
            parser.expect_operator(Operator::RParen);
            Some(Expression::FunctionCall {
                name,
                args: parser.wrap_span(args, start),
            })
        } else {
            Some(variable_mapper.access_variable(parser.reporter, &name, name.clone()))
        }
    } else {
        None
    }
}
