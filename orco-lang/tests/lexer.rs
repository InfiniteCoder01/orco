mod parser_utils;
use parser_utils::*;

#[test]
fn ident() {
    parse("ident", |mut parser| {
        check!(parser.next() == Some(Token::Ident(Span::new("ident"))));
        check!(parser.reporter.len() == 0);
    });
    parse("digits2", |mut parser| {
        check!(parser.next() == Some(Token::Ident(Span::new("digits2"))));
        check!(parser.reporter.len() == 0);
    });
    parse("underscore_2_4_abc", |mut parser| {
        check!(parser.next() == Some(Token::Ident(Span::new("underscore_2_4_abc"))));
        check!(parser.reporter.len() == 0);
    });
    parse("_privateCapital", |mut parser| {
        check!(parser.next() == Some(Token::Ident(Span::new("_privateCapital"))));
        check!(parser.reporter.len() == 0);
    });
    parse("r#return", |mut parser| {
        check!(parser.next() == Some(Token::Ident(Span::new("return"))));
        check!(parser.reporter.len() == 0);
    });
}

#[test]
fn number() {
    parse("42", |mut parser| {
        check!(
            parser.next()
                == Some(Token::Constant(orco::ir::expression::Constant::Integer {
                    value: 42,
                    r#type: orco::ir::Type::IntegerWildcard,
                    metadata: Box::new(()),
                }))
        );
        check!(parser.reporter.len() == 0);
    });
    parse("340282366920938463463374607431768211456", |mut parser| {
        parser.next();
        check!(parser.reporter.len() == 1);
    });
    parse("123_456_789", |mut parser| {
        check!(
            parser.next()
                == Some(Token::Constant(orco::ir::expression::Constant::Integer {
                    value: 123_456_789,
                    r#type: orco::ir::Type::IntegerWildcard,
                    metadata: Box::new(()),
                }))
        );
        check!(parser.reporter.len() == 0);
    });
    parse("0b00101010", |mut parser| {
        check!(
            parser.next()
                == Some(Token::Constant(orco::ir::expression::Constant::Integer {
                    value: 0b00101010,
                    r#type: orco::ir::Type::IntegerWildcard,
                    metadata: Box::new(()),
                }))
        );
        check!(parser.reporter.len() == 0);
    });
    parse("0o1741", |mut parser| {
        check!(
            parser.next()
                == Some(Token::Constant(orco::ir::expression::Constant::Integer {
                    value: 0o1741,
                    r#type: orco::ir::Type::IntegerWildcard,
                    metadata: Box::new(()),
                }))
        );
        check!(parser.reporter.len() == 0);
    });
}
