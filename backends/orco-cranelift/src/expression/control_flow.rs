use super::*;

impl Object {
    pub fn build_return(
        &mut self,
        builder: &mut FunctionBuilder,
        expr: &dyn orco::expression::Return,
    ) -> Option<Value> {
        let value = self.build_expression(builder, expr.expression());
        builder.ins().return_(&if let Some(value) = value {
            vec![value]
        } else {
            vec![]
        });
        None
    }
}
