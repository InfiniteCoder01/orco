use parsel::ast::{Paren, Word};

use super::*;

/// C function definition
#[derive(Clone, PartialEq, Eq, Parse, ToTokens)]
pub struct FunctionDefinition {
    /// Return type
    pub return_type: Type,
    /// Name
    pub name: Word,
    /// Args
    pub args: Paren<kw::void>,
    /// Body
    pub body: statement::Block,
}
