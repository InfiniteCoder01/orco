use orco::ir;
use orco_lang::lexer::Lexer;
use orco_lang::parser;

#[test]
fn types() {
    use std::num::NonZeroU16;
    assert_eq!(
        parser::TypeParser::new().parse(Lexer::new("i8")).unwrap(),
        ir::Type::Int(NonZeroU16::new(1).unwrap())
    );
    assert_eq!(
        parser::TypeParser::new().parse(Lexer::new("i128")).unwrap(),
        ir::Type::Int(NonZeroU16::new(16).unwrap())
    );
    assert_eq!(
        parser::TypeParser::new().parse(Lexer::new("u16")).unwrap(),
        ir::Type::Unsigned(NonZeroU16::new(2).unwrap())
    );
    assert_eq!(
        parser::TypeParser::new().parse(Lexer::new("f32")).unwrap(),
        ir::Type::Float(NonZeroU16::new(4).unwrap())
    );
    assert_eq!(
        parser::TypeParser::new().parse(Lexer::new("bool")).unwrap(),
        ir::Type::Bool
    );
    assert_eq!(
        parser::TypeParser::new().parse(Lexer::new("char")).unwrap(),
        ir::Type::Char
    );
    assert_eq!(
        parser::TypeParser::new()
            .parse(Lexer::new("Custom"))
            .unwrap(),
        ir::Type::Custom("Custom".to_owned())
    );
}
