#![warn(missing_docs)]
//! OrCo language, IR for OrCo compiler toolchain
use lalrpop_util::lalrpop_mod;
use orco::*;

lalrpop_mod!(pub orco_parser);

/// A compilation unit
pub struct Crate {}

impl Crate {
    pub fn parse(path: impl AsRef<std::path::Path>, codebase: &Codebase) -> Self {
        todo!("{}", orco_parser::TermParser::new().parse(&std::fs::read_to_string(path).unwrap()).unwrap())
    }
}

// impl Unit for Crate {
//     //
// }
