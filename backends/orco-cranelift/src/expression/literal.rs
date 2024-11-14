use super::*;

impl Object {
    pub fn build_literal(
        &mut self,
        builder: &mut cl::FunctionBuilder,
        lit: orco::expression::Literal,
    ) -> Option<cl::Value> {
        match lit {
            orco::expression::Literal::Integer(lit) => self.build_integer_literal(builder, lit),
        }
    }

    pub fn build_integer_literal(
        &mut self,
        builder: &mut cl::FunctionBuilder,
        lit: &dyn orco::expression::literal::IntegerLiteral,
    ) -> Option<cl::Value> {
        Some(builder.ins().iconst(
            self.convert_type(lit.r#type())[0].value_type,
            lit.value() as i64,
        ))
    }
}
