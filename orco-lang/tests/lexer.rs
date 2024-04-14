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
}
