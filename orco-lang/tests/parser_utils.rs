#![allow(unused_imports)]
pub use assert2::*;
pub use orco::ir;
pub use orco::Span;
pub use orco_lang::lexer::*;
pub use orco_lang::parser;
pub use std::num::NonZeroU16;

pub fn parse(input: &str, callback: impl FnOnce(Parser<Vec<orco::Report>>)) {
    callback(Parser::new(
        &orco::Src::new(input.to_owned(), "<buffer>".into()),
        &mut Vec::new(),
    ));
}
