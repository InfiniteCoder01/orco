use super::*;
use parsel::{
    ast::{Maybe, Separated, Word},

#[derive(Clone, PartialEq, Eq, Parse, ToTokens)]
pub struct VariableDeclarationEntry {
    pub name: Word,
    pub value: Maybe<Eq, Expression>,
}

    pub op_semi: Semi,
}
