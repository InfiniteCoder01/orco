mod parser_utils;
use parser_utils::*;

#[test]
fn types() {
    use ir::Type;
    use parser::r#type::parse as parse_type;
    parse("i8", |mut parser| {
        assert_eq!(
            parse_type(&mut parser).inner,
            Type::Int(NonZeroU16::new(1).unwrap())
        );
        assert_eq!(parser.reporter.len(), 0);
    });
    parse("i128", |mut parser| {
        assert_eq!(
            parse_type(&mut parser).inner,
            Type::Int(NonZeroU16::new(16).unwrap())
        );
        assert_eq!(parser.reporter.len(), 0);
    });
    parse("u16", |mut parser| {
        assert_eq!(
            parse_type(&mut parser).inner,
            Type::Unsigned(NonZeroU16::new(2).unwrap())
        );
        assert_eq!(parser.reporter.len(), 0);
    });
    parse("f32", |mut parser| {
        assert_eq!(
            parse_type(&mut parser).inner,
            Type::Float(NonZeroU16::new(4).unwrap())
        );
        assert_eq!(parser.reporter.len(), 0);
    });
    parse("bool", |mut parser| {
        assert_eq!(parse_type(&mut parser).inner, Type::Bool);
        assert_eq!(parser.reporter.len(), 0);
    });
    parse("char*", |mut parser| {
        assert_eq!(
            parse_type(&mut parser).inner,
            Type::Pointer(Box::new(ir::Type::Char))
        );
        assert_eq!(parser.reporter.len(), 0);
    });
    parse("Custom", |mut parser| {
        assert_eq!(
            parse_type(&mut parser).inner,
            Type::Custom("Custom".to_owned())
        );
        assert_eq!(parser.reporter.len(), 0);
    });
}

#[test]
fn function() {
    parse(
        "main(argc: u32, argv: char**) -> i32 { return 42; }",
        |mut parser| {
            let function = parser::item::function::parse_named(&mut parser).unwrap();
            assert_eq!(function.name, "main");
            assert_eq!(function.signature.args.len(), 2);
            assert_eq!(function.signature.args[0].lock().unwrap().name.inner, "argc");
            assert_eq!(
                function.signature.args[0].lock().unwrap().r#type.inner,
                ir::Type::Unsigned(NonZeroU16::new(4).unwrap())
            );
            assert_eq!(function.signature.args[1].lock().unwrap().name.inner, "argv");
            assert_eq!(
                function.signature.args[1].lock().unwrap().r#type.inner,
                ir::Type::Pointer(Box::new(ir::Type::Pointer(Box::new(ir::Type::Char))))
            );
            assert_eq!(
                function.signature.return_type.inner,
                ir::Type::Int(NonZeroU16::new(4).unwrap())
            );
            let body = function.body.lock().unwrap();
            assert_eq!(body.expressions.len(), 1);
        },
    );
}
