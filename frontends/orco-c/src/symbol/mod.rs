use super::*;

/// C function declaration and definition
pub mod function;
pub use function::FunctionDefinition;

/// C symbols
#[derive(PartialEq, Eq, Parse, ToTokens)]
pub enum Symbol {
    /// Function definition
    FunctionDefinition(SymbolBox<FunctionDefinition>),
}
