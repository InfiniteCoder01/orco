use super::*;
use parsel::ast::LitInt;

#[derive(Clone, PartialEq, Eq, Parse, ToTokens)]
pub enum Expression {
    Integer(LitInt),
    Variable(SymbolReference),
}

#[derive(Clone, PartialEq, Eq, Parse, ToTokens)]
pub enum SymbolReference {
    Unresolved(parsel::ast::Word),
    // Function(SymbolRef<FunctionDefinition>),
}
