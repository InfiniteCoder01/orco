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

impl orco::expression::VariableDeclaration for VariableDeclaration {
    fn variables(
        &self,
    ) -> orco::DynIter<(orco::Type, std::borrow::Cow<str>, Option<orco::Expression>)> {
        let ty = self.ty.as_orco();
        Box::new(self.variables.iter().map(move |entry| {
            (
                ty.clone(),
                entry.name.to_string().into(),
                entry.value.as_ref().map(|value| value.as_orco()),
            )
        }))
    }

    fn variables_mut(
        &mut self,
    ) -> orco::DynIter<(
        orco::Type,
        std::borrow::Cow<str>,
        Option<orco::Expression<orco::Mut>>,
    )> {
        let ty = self.ty.as_orco();
        Box::new(self.variables.iter_mut().map(move |entry| {
            (
                ty.clone(),
                entry.name.to_string().into(),
                entry.value.as_mut().map(|value| value.as_orco_mut()),
            )
        }))
    }
}
