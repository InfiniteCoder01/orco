use cranelift_codegen::ir::{InstBuilder, Value};
use cranelift_frontend::FunctionBuilder;

pub mod pool;

impl crate::Object<'_> {
    pub fn build_constant(
        &mut self,
        builder: &mut FunctionBuilder,
        value: &orco::ir::expression::Constant,
    ) -> Option<Value> {
        match value {
            orco::ir::expression::Constant::SignedInteger { value, r#type } => {
                self.integer_constant(builder, *value, r#type)
            }
            orco::ir::expression::Constant::UnsignedInteger { value, r#type } => self
                .integer_constant(
                    builder,
                    unsafe { std::mem::transmute::<u128, i128>(*value) },
                    r#type,
                ),
            orco::ir::expression::Constant::CString(bytes) => {
                Some(self.add_constant_to_pool(builder, bytes))
            }
        }
    }

    fn integer_constant(
        &mut self,
        builder: &mut FunctionBuilder,
        value: i128,
        r#type: &orco::ir::types::Type,
    ) -> Option<Value> {
        Some(builder.ins().iconst(self.convert_type(r#type), value as i64))
    }
}
