use cranelift_codegen::ir::{InstBuilder, Value};
use cranelift_frontend::FunctionBuilder;
use cranelift_module::Module;

pub mod block;
pub mod constant;

impl crate::Object {
    pub fn build_expression(
        &mut self,
        builder: &mut FunctionBuilder,
        expression: &orco::ir::Expression,
    ) -> Option<Value> {
        match expression {
            orco::ir::Expression::Block(block) => self.build_block(builder, block),
            orco::ir::Expression::Return(value) => {
                let ret = self.build_expression(builder, value);
                builder.ins().return_(&ret.into_iter().collect::<Vec<_>>());
                None
            }
            orco::ir::expression::Expression::Constant(value) => {
                self.build_constant(builder, value)
            }
            orco::ir::expression::Expression::FunctionCall { name } => {
                let function = self
                    .object
                    .declare_func_in_func(*self.functions.get(name).unwrap(), builder.func);
                let instruction = builder.ins().call(function, &[]);
                builder.inst_results(instruction).first().copied()
            }
        }
    }
}
