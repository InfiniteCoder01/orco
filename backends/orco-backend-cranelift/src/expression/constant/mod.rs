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
            orco::ir::expression::Constant::SignedInteger { value, size } => {
                self.integer_constant(builder, *value, *size)
            }
            orco::ir::expression::Constant::UnsignedInteger { value, size } => self
                .integer_constant(
                    builder,
                    unsafe { std::mem::transmute::<u128, i128>(*value) },
                    *size,
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
        size: Option<std::num::NonZeroU16>,
    ) -> Option<Value> {
        Some(builder.ins().iconst(
        match size.map(std::num::NonZeroU16::get) {
            Some(1) => cranelift_codegen::ir::types::I8,
            Some(2) => cranelift_codegen::ir::types::I16,
            Some(4) => cranelift_codegen::ir::types::I32,
            Some(8) => cranelift_codegen::ir::types::I64,
            Some(16) => todo!("128 bit constant doesn't fit into immediate"),
            None => panic!("Integer constant type is unknown. Cranelift backend requires type inference and type checking to be done beforehand"),
            _ => panic!("Invalid integer size: {:?}", size),
        },
        value as i64,
    ))
    }
}
