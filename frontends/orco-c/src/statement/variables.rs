use super::*;
use parsel::{
    ast::{Maybe, Separated, Word},
    syn::token::{Comma, Eq},
};

#[derive(Clone, PartialEq, Eq, Parse, ToTokens)]
pub struct VariableDeclarationEntry {
    pub name: Word,
    pub value: Maybe<Eq, Expression>,
}

#[derive(Clone, PartialEq, Eq, Parse, ToTokens)]
pub struct VariableDeclaration {
    pub ty: Type,
    pub variables: Separated<VariableDeclarationEntry, Comma>,
    pub op_semi: Semi,
}
