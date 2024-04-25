use orco_lang::lexer::*;

#[test]
fn ident() {
    assert_eq!(
        Parser::new("ident").next(),
        Some(Token::Ident("ident".to_owned()))
    );
    assert_eq!(
        Parser::new("digits2").next(),
        Some(Token::Ident("digits2".to_owned()))
    );
    assert_eq!(
        Parser::new("underscore_2_4_abc").next(),
        Some(Token::Ident("underscore_2_4_abc".to_owned()))
    );
    assert_eq!(
        Parser::new("_privateCapital").next(),
        Some(Token::Ident("_privateCapital".to_owned()))
    );
    assert_eq!(
        Parser::new("r#return").next(),
        Some(Token::Ident("return".to_owned()))
    );
}

#[test]
fn number() {
    assert_eq!(
        Parser::new("42").next(),
        Some(Token::Constant(
            orco::ir::expression::Constant::UnsignedInteger {
                value: 42,
                size: None
            }
        ))
    );
    assert_eq!(
        Parser::new("-128").next(),
        Some(Token::Constant(
            orco::ir::expression::Constant::SignedInteger {
                value: -128,
                size: None
            }
        ))
    );
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
    // assert_eq!(
    //     Parser::new("-170141183460469231731687303715884105729").next(),
    //     Some(Err(Error::IntegerOutOfBounds(
    //         "-170141183460469231731687303715884105729".to_owned()
    //     )))
    // );
    assert_eq!(
        Parser::new("123_456_789").next(),
        Some(Token::Constant(
            orco::ir::expression::Constant::UnsignedInteger {
                value: 123_456_789,
                size: None
            }
        ))
    );
    assert_eq!(
        Parser::new("0b00101010").next(),
        Some(Token::Constant(
            orco::ir::expression::Constant::UnsignedInteger {
                value: 0b00101010,
                size: None
            }
        ))
    );
    assert_eq!(
        Parser::new("0o1741").next(),
        Some(Token::Constant(
            orco::ir::expression::Constant::UnsignedInteger {
                value: 0o1741,
                size: None
            }
        ))
    );
    assert_eq!(
        Parser::new("-0xdeadbeef").next(),
        Some(Token::Constant(
            orco::ir::expression::Constant::SignedInteger {
                value: -0xdeadbeef,
                size: None
            }
        ))
    );
}
