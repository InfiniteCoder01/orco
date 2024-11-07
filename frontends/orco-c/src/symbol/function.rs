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
    pub args: Paren<kw::Void>,
    /// Body
    pub body: statement::Block,
}

impl orco::symbol::Function for FunctionDefinition {
    fn name(&self) -> std::borrow::Cow<str> {
        self.name.to_string().into()
    }

    fn body(&self) -> orco::Expression {
        orco::Expression::Block(&self.body as _)
    }

    fn body_mut(&mut self) -> orco::Expression<orco::Mut> {
        orco::Expression::Block(&mut self.body as _)
    }
}
