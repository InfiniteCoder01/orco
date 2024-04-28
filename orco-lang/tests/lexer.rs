use orco_lang::lexer::*;

fn parse(input: &str, callback: impl FnOnce(Parser)) {
    callback(Parser::new(
        &Source(orco::Src::new(input.to_owned(), "<buffer>".into())),
        &mut orco::diagnostics::DefaultReporter::default(),
    ));
}

#[test]
fn ident() {
    parse("ident", |mut parser| {
        assert_eq!(parser.next(), Some(Token::Ident("ident".to_owned())))
    });
    parse("digits2", |mut parser| {
        assert_eq!(parser.next(), Some(Token::Ident("digits2".to_owned())))
    });
    parse("underscore_2_4_abc", |mut parser| {
        assert_eq!(
            parser.next(),
            Some(Token::Ident("underscore_2_4_abc".to_owned()))
        )
    });
    parse("_privateCapital", |mut parser| {
        assert_eq!(
            parser.next(),
            Some(Token::Ident("_privateCapital".to_owned()))
        )
    });
    parse("r#return", |mut parser| {
        assert_eq!(parser.next(), Some(Token::Ident("return".to_owned())))
    });
}

#[test]
fn number() {
    parse("42", |mut parser| {
        assert_eq!(
            parser.next(),
            Some(Token::Constant(
                orco::ir::expression::Constant::UnsignedInteger {
                    value: 42,
                    r#type: orco::ir::Type::Wildcard,
                }
            ))
        )
    });
    parse("-128", |mut parser| {
        assert_eq!(
            parser.next(),
            Some(Token::Constant(
                orco::ir::expression::Constant::SignedInteger {
                    value: -128,
                    r#type: orco::ir::Type::Wildcard,
                }
            ))
        )
    });
    // assert_eq!(
    //     {
    //         let mut parser = Parser::new("340282366920938463463374607431768211456");
    //         parser.next();
    //         parser.errors
    //     },
    //     Some(Err(Error::IntegerOutOfBounds(
    //         "340282366920938463463374607431768211456".to_owned()
    //     )))
    // );
    parse("-170141183460469231731687303715884105729", |mut parser| {
        assert_eq!(
            parser.next(),
            None // Some(Err(Error::IntegerOutOfBounds(
                 //     "-170141183460469231731687303715884105729".to_owned()
                 // )))
        )
    });
    parse("123_456_789", |mut parser| {
        assert_eq!(
            parser.next(),
            Some(Token::Constant(
                orco::ir::expression::Constant::UnsignedInteger {
                    value: 123_456_789,
                    r#type: orco::ir::Type::Wildcard
                }
            ))
        )
    });
    parse("0b00101010", |mut parser| {
        assert_eq!(
            parser.next(),
            Some(Token::Constant(
                orco::ir::expression::Constant::UnsignedInteger {
                    value: 0b00101010,
                    r#type: orco::ir::Type::Wildcard
                }
            ))
        )
    });
    parse("0o1741", |mut parser| {
        assert_eq!(
            parser.next(),
            Some(Token::Constant(
                orco::ir::expression::Constant::UnsignedInteger {
                    value: 0o1741,
                    r#type: orco::ir::Type::Wildcard
                }
            ))
        )
    });
    parse("-0xdeadbeef", |mut parser| {
        assert_eq!(
            parser.next(),
            Some(Token::Constant(
                orco::ir::expression::Constant::SignedInteger {
                    value: -0xdeadbeef,
                    r#type: orco::ir::Type::Wildcard
                }
            ))
        )
    });
}
