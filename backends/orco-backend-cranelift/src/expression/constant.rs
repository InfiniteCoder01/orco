use cranelift_codegen::ir::{InstBuilder, Value};
use cranelift_frontend::FunctionBuilder;
use cranelift_module::Module;

impl crate::Object {
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
                let id = self
                    .object
                    .declare_data("data", cranelift_module::Linkage::Hidden, false, false)
                    .unwrap();
                self.object
                    .define_data(
                        id,
                        &cranelift_module::DataDescription {
                            init: cranelift_module::Init::Bytes {
                                contents: bytes.as_slice().into(),
                            },
                            function_decls: Default::default(),
                            data_decls: Default::default(),
                            function_relocs: Default::default(),
                            data_relocs: Default::default(),
                            custom_segment_section: Default::default(),
                            align: Default::default(),
                        },
                    )
                    .unwrap();

                let local_id = self.object.declare_data_in_func(id, builder.func);
                let pointer = self.object.target_config().pointer_type();
                let local_symbol = builder.ins().symbol_value(pointer, local_id);
                Some(local_symbol)
            }
        }
    }

    fn integer_constant(
        &mut self,
        builder: &mut FunctionBuilder,
        value: i128,
        size: Option<u16>,
    ) -> Option<Value> {
        Some(builder.ins().iconst(
        match size {
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
