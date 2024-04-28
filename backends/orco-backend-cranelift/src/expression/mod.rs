use cranelift_codegen::ir::{InstBuilder, Value};
use cranelift_frontend::FunctionBuilder;
use cranelift_module::Module;

pub mod block;
pub mod constant;

impl crate::Object<'_> {
    pub fn build_expression(
        &mut self,
        builder: &mut FunctionBuilder,
        expression: &orco::ir::Expression,
    ) -> Option<Value> {
        match expression {
            orco::ir::expression::Expression::Constant(value) => {
                self.build_constant(builder, value)
            }
            orco::ir::expression::Expression::BinaryOp(lhs, op, rhs) => {
                let lhs = self.build_expression(builder, lhs)?;
                let rhs = self.build_expression(builder, rhs)?;
                match op {
                    orco::ir::expression::BinaryOp::Add => Some(builder.ins().iadd(lhs, rhs)),
                    orco::ir::expression::BinaryOp::Sub => Some(builder.ins().isub(lhs, rhs)),
                    orco::ir::expression::BinaryOp::Mul => Some(builder.ins().imul(lhs, rhs)),
                    orco::ir::expression::BinaryOp::Div => Some(builder.ins().sdiv(lhs, rhs)),
                    orco::ir::expression::BinaryOp::Mod => Some(builder.ins().srem(lhs, rhs)),
                }
            }
            orco::ir::Expression::Block(block) => self.build_block(builder, block),
            orco::ir::expression::Expression::FunctionCall { name, args } => {
                let function = self.object.declare_func_in_func(
                    *self
                        .functions
                        .get(&name.inner)
                        .unwrap_or_else(|| panic!("Function {} is not defined", name.inner)),
                    builder.func,
                );
                let args = args.iter().map(|arg| self.build_expression(builder, arg).expect("Can't pass a unit type as an argument to a function, did you run type checking/inference?")).collect::<Vec<_>>();
                let instruction = builder.ins().call(function, &args);
                builder.inst_results(instruction).first().copied()
            }
            orco::ir::Expression::Return(value) => {
                let ret = self.build_expression(builder, value);
                builder.ins().return_(&ret.into_iter().collect::<Vec<_>>());
                builder.seal_block(builder.current_block().unwrap());
                None
            }
            orco::ir::Expression::VariableDeclaration { .. } => {
                todo!()
            }
            orco::ir::Expression::Error(span) => panic!("IR contains errors at {:?}!", span),
        }
    }
}
