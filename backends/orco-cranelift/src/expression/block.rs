use super::*;

impl Object {
    pub fn build_block(
        &mut self,
        builder: &mut FunctionBuilder,
        block: &dyn orco::expression::Block,
    ) -> Option<Value> {
        for expression in block.expressions() {
            self.build_expression(builder, expression);
        }
        None
    }
}
