#![doc = include_str!("../README.md")]
use parsel::ast::Many;
use parsel::{Parse, ToTokens};

pub use parsel;

/// Wrapper around [`orco::SymbolBox`] with parsing traits
pub mod symbol_box;
pub use symbol_box::SymbolBox;

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
        return => Return;
        int => Int;
        void => Void;
    }
}

/// Translation unit
#[derive(PartialEq, Eq, Parse, ToTokens)]
pub struct Unit {
    pub symbols: Many<Symbol>,
}

impl orco::Unit for Unit {
    fn symbols(&self) -> orco::DynIter<orco::Symbol> {
        Box::new(self.symbols.iter().map(|symbol| symbol.as_orco()))
    }
}

#[test]
pub fn parse_test() {
    use assert2::*;
    let unit: Unit = parsel::parse_quote! {
        int main(void) {
            return 42;
        }
    };
    check!(unit.symbols.len() == 1);
    let main = unit.symbols.first().unwrap();
    let_assert!(Symbol::FunctionDefinition(main) = main);
    let main = main.object().try_read().unwrap();
    check!(let Type::Int(_) = main.return_type);
    check!(main.name == "main");
    check!(main.body.0.len() == 1);
    let_assert!(Some(Statement::Return(expr)) = main.body.0.first());
    let_assert!(Expression::Integer(rv) = &expr.expression);
    check!(rv.0.value() == 42);
}
