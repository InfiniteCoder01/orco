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

/// Keywords
pub mod kw {
    #![allow(missing_docs)]
    parsel::custom_keyword!(int);
    parsel::custom_keyword!(void);
}

/// Translation unit
#[derive(PartialEq, Eq, Parse, ToTokens)]
pub struct Unit {
    pub symbols: Many<Symbol>,
}

#[test]
pub fn parse_test() {
    use assert2::*;
    let unit: Unit = parsel::parse_quote! {
        int main(void) {

        }
    };
    check!(unit.symbols.len() == 1);
    let main = unit.symbols.first().unwrap();
    let_assert!(Symbol::FunctionDefinition(main) = main);
    let main = main.object().try_read().unwrap();
    check!(let Type::Int(_) = main.return_type);
    check!(main.name == "main");
}
