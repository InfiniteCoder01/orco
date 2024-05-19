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
    pub fn parse(
        path: impl AsRef<std::path::Path>,
        reporter: &mut dyn orco::diagnostics::ErrorReporter,
    ) -> Self {
        Self {
            root: parser::parse(&mut lexer::Parser::new(
                &lexer::Source(orco::Src::load(path.as_ref().to_path_buf()).unwrap()),
                reporter,
            )),
        }
    }
}

/// Symbol resolver for OrCo IR language
pub fn symbol_resolver(
    type_inference: &mut orco::TypeInference,
    path: &orco::Path,
) -> Option<orco::SymbolReference> {
    let start = path.0.first().expect("Trying to resolve an empty path!");
    if let Some(symbol) = type_inference.get_symbol(start) {
        return Some(symbol);
    }
    if let Some(symbol) = type_inference.current_module.symbol_map.get(start) {
        return Some(symbol.first().unwrap().clone());
    }
    None
}
