mod parser_utils;
use parser_utils::*;

#[test]
fn types() {
    use ir::Type;
    use parser::r#type::parse as parse_type;
    parse("i8", |mut parser| {
        check!(parse_type(&mut parser).inner == Type::Int(NonZeroU16::new(1).unwrap()));
        check!(parser.reporter.is_empty());
    });
    parse("i128", |mut parser| {
        check!(parse_type(&mut parser).inner == Type::Int(NonZeroU16::new(16).unwrap()));
        check!(parser.reporter.is_empty());
    });
    parse("u16", |mut parser| {
        check!(parse_type(&mut parser).inner == Type::Unsigned(NonZeroU16::new(2).unwrap()));
        check!(parser.reporter.is_empty());
    });
    parse("f32", |mut parser| {
        check!(parse_type(&mut parser).inner == Type::Float(NonZeroU16::new(4).unwrap()));
        check!(parser.reporter.is_empty());
    });
    parse("bool", |mut parser| {
        check!(parse_type(&mut parser).inner == Type::Bool);
        check!(parser.reporter.is_empty());
    });
    parse("*char", |mut parser| {
        check!(parse_type(&mut parser).inner == Type::Pointer(Box::new(ir::Type::Char), false));
        check!(parser.reporter.is_empty());
    });
    parse("*mut char", |mut parser| {
        check!(parse_type(&mut parser).inner == Type::Pointer(Box::new(ir::Type::Char), true));
        check!(parser.reporter.is_empty());
    });
    parse("Custom", |mut parser| {
        check!(parse_type(&mut parser).inner == Type::Custom(Span::new("Custom")));
        check!(parser.reporter.is_empty());
    });
}

#[test]
fn function() {
    parse("fn (argc: u32, argv: **char) -> i32 42", |mut parser| {
        let function = parser::expression::parse(&mut parser);
        let_assert!(Some(orco::ir::Expression::Function(function)) = function);
        check!(function.signature.args.len() == 2);
        check!(function.signature.args[0].name == Span::new("argc"));
        check!(
            *function.signature.args[0].r#type.try_lock().unwrap()
                == ir::Type::Unsigned(NonZeroU16::new(4).unwrap())
        );
        check!(function.signature.args[1].name == Span::new("argv"));
        check!(
            *function.signature.args[1].r#type.try_lock().unwrap()
                == ir::Type::Pointer(
                    Box::new(ir::Type::Pointer(Box::new(ir::Type::Char), false)),
                    false
                )
        );
        check!(function.signature.return_type.inner == ir::Type::Int(NonZeroU16::new(4).unwrap()));
        let body = function.body.try_lock().unwrap();
        let_assert!(ir::Expression::Constant(expr) = &*body);
        check!(
            expr.inner
                == ir::expression::Constant::Integer {
                    value: 42,
                    r#type: ir::Type::IntegerWildcard,
                    metadata: Box::new(()),
                }
        );
    });
}
