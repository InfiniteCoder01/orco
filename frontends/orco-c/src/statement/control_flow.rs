use super::*;

#[derive(Parse, ToTokens)]
pub struct Return {
    pub kw_return: kw::Return,
    pub expression: Expression,
    pub op_semi: Semi,
}
impl Return {
    pub fn build(
        &self,
        ctx: &mut orco::TypeInferenceContext,
        expressions: &mut Vec<orco::Expression>,
    ) {
        let expr = self.expression.build(ctx, expressions);
        if let Some(function) = ctx.intrinsics.r#return.clone() {
            expressions.push(orco::Expression::Call(orco::expression::Call {
                function,
                args: vec![expr],
            }));
        } else {
            todo!("Error")
        }
    }
}
