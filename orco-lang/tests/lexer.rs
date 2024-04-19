use orco_lang::lexer::*;

#[test]
fn ident() {
    assert_eq!(
        Lexer::new("ident").next().unwrap().unwrap().1,
        Token::Ident("ident".to_owned())
    );
    assert_eq!(
        Lexer::new("digits2").next().unwrap().unwrap().1,
        Token::Ident("digits2".to_owned())
    );
    assert_eq!(
        Lexer::new("underscore_2_4_abc").next().unwrap().unwrap().1,
        Token::Ident("underscore_2_4_abc".to_owned())
    );
    assert_eq!(
        Lexer::new("_privateCapital").next().unwrap().unwrap().1,
        Token::Ident("_privateCapital".to_owned())
    );
    assert_eq!(
        Lexer::new("r#return").next().unwrap().unwrap().1,
        Token::Ident("return".to_owned())
    );
}

#[test]
fn number() {
    assert_eq!(
        Lexer::new("42").next().unwrap().unwrap().1,
        Token::Constant(orco::ir::expression::Constant::UnsignedInteger {
            value: 42,
            size: None
        })
    );
    assert_eq!(
        Lexer::new("-128").next().unwrap().unwrap().1,
        Token::Constant(orco::ir::expression::Constant::SignedInteger {
            value: -128,
            size: None
        })
    );
    assert_eq!(
        Lexer::new("340282366920938463463374607431768211456")
            .next()
            .unwrap(),
        Err(Error::IntegerOutOfBounds(
            "340282366920938463463374607431768211456".to_owned()
        ))
    );
    assert_eq!(
        Lexer::new("-170141183460469231731687303715884105729")
            .next()
            .unwrap(),
        Err(Error::IntegerOutOfBounds(
            "-170141183460469231731687303715884105729".to_owned()
        ))
    );
    assert_eq!(
        Lexer::new("123_456_789").next().unwrap().unwrap().1,
        Token::Constant(orco::ir::expression::Constant::UnsignedInteger {
            value: 123_456_789,
            size: None
        })
    );
    assert_eq!(
        Lexer::new("0b00101010").next().unwrap().unwrap().1,
        Token::Constant(orco::ir::expression::Constant::UnsignedInteger {
            value: 0b00101010,
            size: None
        })
    );
    assert_eq!(
        Lexer::new("0o1741").next().unwrap().unwrap().1,
        Token::Constant(orco::ir::expression::Constant::UnsignedInteger {
            value: 0o1741,
            size: None
        })
    );
    assert_eq!(
        Lexer::new("-0xdeadbeef").next().unwrap().unwrap().1,
        Token::Constant(orco::ir::expression::Constant::SignedInteger {
            value: -0xdeadbeef,
            size: None
        })
    );
}
