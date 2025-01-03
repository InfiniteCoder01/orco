use parsel::{
    ast::{Either, Maybe, Paren, Punctuated, Word},
    syn::token::Comma,
};

use super::*;

#[derive(Clone, PartialEq, Eq, Parse, ToTokens)]
pub struct FunctionParameter {
    pub r#type: Type,
    pub name: Maybe<Word>,
}

impl orco::symbol::function::FunctionParameter for FunctionParameter {
    fn name(&self) -> Option<orco::CowStr> {
        self.name.as_prefix().map(|name| name.to_string().into())
    }

    fn r#type(&self) -> orco::Type {
        self.r#type.as_orco()
    }
}

#[derive(Parse, ToTokens)]
pub struct FunctionDefinition {
    pub return_type: Type,
    pub name: Word,
    pub params: Paren<Either<kw::Void, Punctuated<FunctionParameter, Comma>>>,
    pub body: statement::Block,
}

impl orco::symbol::Function for FunctionDefinition {
    fn name(&self) -> orco::CowStr {
        self.name.to_string().into()
    }

    fn signature(&self) -> &dyn orco::symbol::function::FunctionSignature {
        self
    }

    fn signature_mut(&mut self) -> &mut dyn orco::symbol::function::FunctionSignature {
        self
    }

    fn body(&self) -> orco::Expression {
        orco::Expression::Block(&self.body as _)
    }

    fn body_mut(&mut self) -> orco::Expression<orco::Mut> {
        orco::Expression::Block(&mut self.body as _)
    }
}

impl orco::symbol::function::FunctionSignature for FunctionDefinition {
    fn parameters(&self) -> orco::DynIter<&dyn orco::symbol::function::FunctionParameter> {
        match self.params.as_ref() {
            Either::Left(_) => Box::new(std::iter::empty()),
            Either::Right(params) => Box::new(params.iter().map(|param| param as _)),
        }
    }

    fn parameters_mut(
        &mut self,
    ) -> orco::DynIter<&mut dyn orco::symbol::function::FunctionParameter> {
        match self.params.as_mut() {
            Either::Left(_) => Box::new(std::iter::empty()),
            Either::Right(params) => Box::new(params.iter_mut().map(|param| param as _)),
        }
    }

    fn return_type(&self) -> orco::Type {
        self.return_type.as_orco()
    }
}
