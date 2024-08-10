use super::*;

impl crate::Object {
    /// Build a code block
    pub fn build_block(
        &mut self,
        builder: &mut FunctionBuilder,
        block: &orco::ir::expression::Block,
    ) -> Option<Value> {
        for expression in &block.expressions {
            self.build_expression(builder, expression);
            if expression.get_type() == orco::ir::Type::Never {
                return None;
            }
        }
        block
            .tail_expression
            .as_ref()
            .and_then(|expr| self.build_expression(builder, expr))
    }
}
