#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

/// Lexer (splits input into tokens)
pub mod lexer;

/// Hand-written parser
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
            root: parser::parse(&mut lexer::Parser::new(
                &lexer::Source(orco::Src::load(path.as_ref().to_path_buf()).unwrap()),
                Box::new(orco::diagnostics::DefaultReporter::default()),
            )),
        }
    }
}
