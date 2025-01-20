use super::*;
use parsel::ast::{Maybe, Paren};

#[derive(Parse, ToTokens)]
pub struct If {
    pub kw_if: kw::If,
    pub condition: Paren<Expression>,
    #[parsel(recursive)]
    pub then_block: Statement,
    #[parsel(recursive)]
    pub else_block: Maybe<kw::Else, Statement>,
}

impl If {
    pub fn build(
        &self,
        ctx: &mut orco::TypeInferenceContext,
        expressions: &mut Vec<orco::Expression>,
    ) {
        let condition = self.condition.build(ctx, expressions);
        let then_block = {
            let mut expressions = Vec::new();
            self.then_block.build(ctx, &mut expressions);
            orco::expression::Function::new(
                orco::function_signature![() -> () transparent],
                None,
                expressions,
            )
        };
        let else_block = {
            let mut expressions = Vec::new();
            if let Some(else_block) = self.else_block.as_ref() {
                else_block.build(ctx, &mut expressions);
            }
            orco::expression::Function::new(
                orco::function_signature![() -> () transparent],
                None,
                expressions,
            )
        };
        expressions.push(orco::Expression::Call(orco::expression::Call {
            function: ctx.intrinsics.branch(orco::Type::Unit),
            args: vec![
                condition,
                orco::Expression::Function(then_block),
                orco::Expression::Function(else_block),
            ],
        }));
    }
}
