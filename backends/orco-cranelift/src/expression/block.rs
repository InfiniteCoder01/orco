use super::*;

impl Object {
    pub fn build_block(
        &mut self,
        builder: &mut cl::FunctionBuilder,
        block: &dyn orco::expression::Block,
    ) -> Option<cl::Value> {
        for expression in block.expressions() {
            self.build_expression(builder, expression);
        }
        None
    }
}
