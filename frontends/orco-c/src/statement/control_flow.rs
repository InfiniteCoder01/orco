use super::*;

#[derive(Parse, ToTokens)]
pub struct Return {
    pub kw_return: kw::Return,
    pub expression: Expression,
    pub op_semi: Semi,
    pub operator: Unparse<Option<Box<dyn orco::operators::Operator>>>,
}

impl orco::operators::AsOperator for Return {
    fn infer_types(&mut self, ctx: &mut orco::TypeInferenceContext) {
        *self.operator = Some(ctx.operator_return(expression));
    }

    fn operator(&self) -> Option<&dyn orco::operators::Operator> {
        self.operator.as_ref().map(Box::as_ref)
    }
}
