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

impl Symbol {
    pub fn as_orco(&self) -> orco::Symbol {
        match self {
            Self::FunctionDefinition(symbol) => orco::Symbol::Function(symbol.object()),
        }
    }
}
