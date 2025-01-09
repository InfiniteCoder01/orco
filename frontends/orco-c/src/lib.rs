#![doc = include_str!("../README.md")]
use parsel::ast::Many;
use parsel::{Parse, ToTokens};

pub use parsel;

pub mod expression;
pub use expression::Expression;

pub mod statement;
pub use statement::Statement;

/// Symbols such as function/variable declarations, definitions, struts, typedefs, constants, etc.
pub mod symbol;
pub use symbol::Symbol;

/// Types
pub mod r#type;
pub use r#type::Type;

parsel::define_keywords! {
    mod kw {
        if => If;
        else => Else;
        return => Return;
        int => Int;
        void => Void;
    }
}

/// Translation unit
#[derive(Parse, ToTokens)]
pub struct Unit {
    pub symbols: Many<Symbol>,
}

impl Unit {
    pub fn build(
        &self,
        ctx: &mut orco::TypeInferenceContext,
    ) -> std::collections::HashMap<String, orco::Expression> {
        let mut symbols = std::collections::HashMap::new();
        for symbol in &self.symbols {
            let (name, symbol) = symbol.build(ctx);
            symbols.insert(name, symbol);
        }
        symbols
    }
}

#[test]
pub fn parse_test() {
    use assert2::*;
    let unit: Unit = parsel::parse_quote! {
        int main(void) {
            return 42;
        }

        void foo(int x) {}
    };
    check!(unit.symbols.len() == 2);
    let main = unit.symbols.first().unwrap();
    let_assert!(Symbol::FunctionDefinition(main) = main);
    check!(let Type::Int(_) = main.return_type);
    check!(main.name == "main");
    check!(main.params.is_left());
    check!(main.body.0.len() == 1);
    let_assert!(Some(Statement::Return(expr)) = main.body.0.first());
    let_assert!(Expression::Literal(expression::Literal::Integer(rv)) = &expr.expression);
    check!(rv.value() == 42);

    let_assert!(Symbol::FunctionDefinition(foo) = &unit.symbols[1]);
    check!(let Type::Void(_) = foo.return_type);
    check!(foo.name == "foo");
    let_assert!(parsel::ast::Either::Right(params) = foo.params.as_ref());
    check!(params.len() == 1);
    check!(let Type::Int(_) = params.first().unwrap().r#type);
    check!(params
        .first()
        .unwrap()
        .name
        .as_prefix()
        .is_some_and(|name| name.to_string() == "x"));
}

// #[test]
// pub fn interface_test() {
//     let unit: Unit = parsel::parse_quote! {
//         int main(void) {
//             return 42;
//         }

//         void foo(int x) {}
//     };

//     let symbols = unit.symbols.iter().map(Symbol::as_orco).collect::<Vec<_>>();

//     orco::test_symbols(
//         &symbols,
//         &[
//             "
//                 fn main () -> i32 {
//                     return 42;
//                 }
//             ",
//             "
//                 fn foo (x: i32) -> () {
//                 }
//             ",
//         ],
//     );
// }
