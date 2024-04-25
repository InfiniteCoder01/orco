use cranelift_codegen::ir::Value;
use cranelift_frontend::FunctionBuilder;

impl crate::Object<'_> {
    pub fn build_block(
        &mut self,
        builder: &mut FunctionBuilder,
        block: &orco::ir::expression::Block,
    ) -> Option<Value> {
        for expression in &block.expressions {
            self.build_expression(builder, expression);
            if expression.get_type(self.root) == orco::ir::Type::Never {
                return None;
            }
        }
        None
    }
}
