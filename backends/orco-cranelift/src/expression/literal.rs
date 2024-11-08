use super::*;

impl Object {
    pub fn build_literal(
        &mut self,
        builder: &mut FunctionBuilder,
        lit: orco::expression::Literal,
    ) -> Option<Value> {
        match lit {
            orco::expression::Literal::Integer(lit) => self.build_integer_literal(builder, lit),
        }
    }

    pub fn build_integer_literal(
        &mut self,
        builder: &mut FunctionBuilder,
        lit: &dyn orco::expression::literal::IntegerLiteral,
    ) -> Option<Value> {
        Some(builder.ins().iconst(types::I32, lit.value() as i64))
    }
}
