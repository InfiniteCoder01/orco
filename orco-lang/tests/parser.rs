use std::cell::RefCell;
use std::num::NonZeroU16;

use orco::ir;
// use orco_lang::lexer::Lexer;
// use orco_lang::parser;

// #[test]
// fn types() {
//     assert_eq!(
//         parser::TypeParser::new().parse(Lexer::new("i8")),
//         Ok(ir::Type::Int(NonZeroU16::new(1).unwrap()))
//     );
//     assert_eq!(
//         parser::TypeParser::new().parse(Lexer::new("i128")),
//         Ok(ir::Type::Int(NonZeroU16::new(16).unwrap()))
//     );
//     assert_eq!(
//         parser::TypeParser::new().parse(Lexer::new("u16")),
//         Ok(ir::Type::Unsigned(NonZeroU16::new(2).unwrap()))
//     );
//     assert_eq!(
//         parser::TypeParser::new().parse(Lexer::new("f32")),
//         Ok(ir::Type::Float(NonZeroU16::new(4).unwrap()))
//     );
//     assert_eq!(
//         parser::TypeParser::new().parse(Lexer::new("bool")),
//         Ok(ir::Type::Bool)
//     );
//     assert_eq!(
//         parser::TypeParser::new().parse(Lexer::new("char")),
//         Ok(ir::Type::Char)
//     );
//     assert_eq!(
//         parser::TypeParser::new().parse(Lexer::new("Custom")),
//         Ok(ir::Type::Custom("Custom".to_owned()))
//     );
// }

// #[test]
// fn function() {
//     assert_eq!(
//         parser::FunctionParser::new().parse(Lexer::new("fn main() -> i32 { return 42; }")),
//         Ok(orco_lang::parser_utils::Named::new(
//             "main".to_owned(),
//             ir::item::function::Function {
//                 signature: ir::item::function::Signature {
//                     args: vec![],
//                     return_type: ir::Type::Int(NonZeroU16::new(4).unwrap()),
//                 },
//                 body: RefCell::new(ir::expression::Block::new(vec![
//                     ir::expression::Expression::Return(Box::new(
//                         ir::expression::Expression::Constant(
//                             ir::expression::Constant::UnsignedInteger {
//                                 value: 42,
//                                 size: None
//                             }
//                         )
//                     ))
//                 ]))
//             }
//         ))
//     );
//     assert_eq!(
//         parser::FunctionParser::new().parse(Lexer::new("fn foo(bar: f32) {}")),
//         Ok(orco_lang::parser_utils::Named::new(
//             "foo".to_owned(),
//             ir::item::function::Function {
//                 signature: ir::item::function::Signature {
//                     args: vec![(
//                         "bar".to_owned(),
//                         ir::Type::Float(NonZeroU16::new(4).unwrap())
//                     )],
//                     return_type: ir::Type::unit(),
//                 },
//                 body: RefCell::default()
//             }
//         ))
//     );
// }
