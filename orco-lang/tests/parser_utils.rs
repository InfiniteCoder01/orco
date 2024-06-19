#![allow(unused_imports)]
pub use orco::ir;
pub use orco::Span;
pub use orco_lang::lexer::*;
pub use orco_lang::parser;
pub use std::num::NonZeroU16;
pub use assert2::*;

pub fn parse(input: &str, callback: impl FnOnce(Parser<Vec<orco::diagnostics::Report>>)) {
    callback(Parser::new(
        &Source(orco::Src::new(input.to_owned(), "<buffer>".into())),
        &mut Vec::new(),
    ));
}
