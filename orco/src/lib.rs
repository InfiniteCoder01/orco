#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![feature(unsize)]

pub use orco_procmacro::*;

/// See [Expression]
pub mod expression;
pub use expression::operators;
pub use expression::Expression;

/// See [Symbol]
pub mod symbol;
pub use symbol::Symbol;

/// See [Type]
pub mod types;
pub use types::Type;

/// Mutability
pub mod mutability;
pub use mutability::*;

/// See [TypeInferenceContext]
pub mod type_inference;
pub use type_inference::TypeInferenceContext;

/// Symbol references are one of the key features of OrCo.
/// They allow symbols to be accessed from anywhere
pub mod symbol_box;
pub use symbol_box::{SymbolBox, SymbolRef};

/// Boxed dynamic iterator
pub type DynIter<'a, T> = Box<dyn Iterator<Item = T> + 'a>;

/// `Cow<str>`
pub type CowStr<'a> = std::borrow::Cow<'a, str>;

/// EXPERIMENTAL API!
/// Use in tests. Checks if unit has all symbols like symbols using [`std::fmt::Display`] formatting
pub fn test_symbols<'a>(symbols: &[Symbol<'a>], expected: &[impl AsRef<str>]) {
    assert2::check!(symbols.len() == expected.len());
    let mut failed_tests = 0;
    for (expected, symbol) in expected.iter().zip(symbols.iter()) {
        let expected = unindent::unindent(expected.as_ref().trim());
        let symbol = unindent::unindent(symbol.to_string().trim());
        let lines = diff::lines(&expected, &symbol);

        if lines
            .iter()
            .any(|diff| !matches!(diff, diff::Result::Both(_, _)))
        {
            for diff in lines {
                match diff {
                    diff::Result::Left(l) => println!("\x1b[91m-{}\x1b[0m", l),
                    diff::Result::Both(l, _) => println!(" {}", l),
                    diff::Result::Right(r) => println!("\x1b[92m+{}\x1b[0m", r),
                }
            }
            failed_tests += 1;
        }
    }

    if failed_tests > 0 {
        panic!("{} tests failed.", failed_tests);
    }
}
