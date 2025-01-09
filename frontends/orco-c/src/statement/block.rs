use super::*;

#[derive(Parse, ToTokens)]
pub struct Block(#[parsel(recursive)] pub Brace<Many<Statement>>);

impl Block {
    pub fn build(
        &self,
        ctx: &mut orco::TypeInferenceContext,
        expressions: &mut Vec<orco::Expression>,
    ) {
        ctx.scopes.push(orco::type_inference::Scope::new());
        for stmt in self.0.iter() {
            stmt.build(ctx, expressions);
        }
        ctx.scopes.pop();
    }
}
