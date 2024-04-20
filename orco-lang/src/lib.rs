//! OrCo language, IR for OrCo compiler toolchain
#![warn(missing_docs)]

use lalrpop_util::lalrpop_mod;

/// Lexer (splits input into tokens)
pub mod lexer;

/// Different utils for the parser
pub mod parser_utils;
// lalrpop_mod!(#[allow(missing_docs)] pub parser);
pub mod parser;

/// A compilation unit
pub struct Crate {
    /// The root module
    pub root: orco::ir::Module,
}

impl Crate {
    /// Parse the crate
    pub fn parse(path: impl AsRef<std::path::Path>) -> Self {
        Self {
            root: parser::ModuleParser::new()
                .parse(lexer::Lexer::new(&std::fs::read_to_string(path).unwrap()))
                .unwrap(),
        }
    }
}
