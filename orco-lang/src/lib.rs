#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

/// Lexer (splits input into tokens)
pub mod lexer;

/// Hand-written parser
pub mod parser;

/// A compilation unit
pub struct Crate {
    /// The root module
    pub root: std::pin::Pin<Box<orco::ir::Module>>,
}

impl Crate {
    /// Parse the crate
    pub fn parse(
        path: impl AsRef<std::path::Path>,
        reporter: &mut dyn orco::diagnostics::ErrorReporter,
    ) -> Self {
        Self::parse_src(
            &orco::Src::load(path.as_ref().to_path_buf()).unwrap(),
            reporter,
        )
    }

    /// Parse the crate from [`orco::Src`]
    pub fn parse_src(src: &orco::Src, reporter: &mut dyn orco::diagnostics::ErrorReporter) -> Self {
        Self {
            root: Box::pin(parser::parse(
                &mut lexer::Parser::new(&src, reporter),
                false,
            )),
        }
    }
}
