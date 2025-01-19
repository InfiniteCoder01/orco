use super::*;
use cranelift::prelude::InstBuilder;

// pub mod block;
// pub mod control_flow;
// pub mod literal;

impl Object {
    pub fn build_expression(
        &mut self,
        builder: &mut cl::FunctionBuilder,
        expression: &orco::Expression,
    ) -> Option<cl::Value> {
        // match expression {
        //     orco::Expression::Block(block) => self.build_block(builder, block),
        //     orco::Expression::Return(expr) => self.build_return(builder, expr),
        //     orco::Expression::VariableDeclaration(_) => todo!(),
        //     orco::Expression::FunctionCall(_) => todo!(),
        //     orco::Expression::Literal(lit) => self.build_literal(builder, lit),
        // }
        match expression {
            orco::Expression::Literal(literal) => todo!(),
            orco::Expression::Variable(rw_lock) => todo!(),
            orco::Expression::Function(function) => todo!(),
            orco::Expression::Call(call) => todo!(),
            orco::Expression::Error => todo!(),
        }
    }
}
