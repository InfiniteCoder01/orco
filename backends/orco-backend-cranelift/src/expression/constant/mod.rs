use cranelift_codegen::ir::{InstBuilder, Value};
use cranelift_frontend::FunctionBuilder;

/// Constant pool
pub mod pool;

impl crate::Object<'_> {
    /// Build a constant expression
    pub fn build_constant(
        &mut self,
        builder: &mut FunctionBuilder,
        value: &orco::ir::expression::Constant,
    ) -> Option<Value> {
        match value {
            orco::ir::expression::Constant::Integer { value, r#type } => {
                if let orco::ir::Type::Int(size) = r#type {
                    if size.get() > 8 {
                        todo!("Cranelift integer constants bigger than 64 bits");
                    }
                }
                Some(
                    builder
                        .ins()
                        .iconst(self.convert_type(r#type), *value as i64),
                )
            }
            orco::ir::expression::Constant::CString(bytes) => {
                Some(self.add_constant_to_pool(builder, bytes))
            }
        }
    }
}
