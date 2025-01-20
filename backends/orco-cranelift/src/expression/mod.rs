use crate::cl;
use cranelift::prelude::InstBuilder;

// pub mod block;
// pub mod control_flow;
// pub mod literal;
pub mod intrinsic;

impl crate::Object {
    pub fn build_expression(
        &mut self,
        builder: &mut cl::FunctionBuilder,
        expression: &orco::Expression,
    ) -> Option<cl::Value> {
        use orco::Expression;
        match expression {
            Expression::Literal(literal) => todo!(),
            Expression::Variable(rw_lock) => todo!(),
            Expression::Function(function) => todo!(),
            Expression::Call(call) => {
                let function = call.function.read().unwrap();
                use orco::types::CallingConvention;
                match function.signature.calling_convention {
                    CallingConvention::Transparent => {
                        match function.body {
                            orco::expression::function::FunctionBody::Block(vec) => todo!(),
                            orco::expression::function::FunctionBody::Intrinsic(intrinsic) => {
                                todo!()
                            }
                            //
                        }
                    }
                    CallingConvention::Inline => todo!(),
                    CallingConvention::Fastest => todo!(),
                    CallingConvention::SystemV => todo!(),
                    CallingConvention::Fastcall => todo!(),
                }
            }
            Expression::Error => todo!(),
        }
    }

    pub fn inline_function(
        &mut self,
        builder: cl::FunctionBuilder,
        function: orco::expression::Function,
        args: Vec<orco::Expression>,
    ) -> Option<cl::Value> {
    }
}
