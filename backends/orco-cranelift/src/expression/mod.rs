use super::*;

pub mod block;
pub mod control_flow;
pub mod literal;

impl Object {
    pub fn build_expression(
        &mut self,
        builder: &mut FunctionBuilder,
        expression: orco::Expression,
    ) -> Option<Value> {
        match expression {
            orco::Expression::Block(block) => self.build_block(builder, block),
            orco::Expression::Return(expr) => self.build_return(builder, expr),
            orco::Expression::Literal(lit) => self.build_literal(builder, lit),
        }
    }
}