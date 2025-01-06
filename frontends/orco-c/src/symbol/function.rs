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

#[derive(Parse, ToTokens)]
pub struct FunctionDefinition {
    pub return_type: Type,
    pub name: Word,
    pub params: Paren<Either<kw::Void, Punctuated<FunctionParameter, Comma>>>,
    pub body: statement::Block,
}

impl FunctionDefinition {
    pub fn build(&self, ctx: &mut orco::TypeInferenceContext) -> orco::expression::Function {
        let mut expressions = Vec::new();
        ctx.enter_function();
        self.body.build(ctx, &mut expressions);
        ctx.exit_function();
        orco::expression::Function::new(Some(self.name.to_string()), expressions)
    }
}
