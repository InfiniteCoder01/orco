use cranelift_module::Module;

impl crate::Object<'_> {
    pub fn convert(&self, r#type: &orco::ir::Type) -> cranelift_codegen::ir::Type {
        match r#type {
            orco::ir::Type::Int(bytes) | orco::ir::Type::Unsigned(bytes) => integer(*bytes),
            orco::ir::Type::Float(bytes) => match bytes.get() {
                4 => cranelift_codegen::ir::types::F32,
                8 => cranelift_codegen::ir::types::F64,
                _ => unimplemented!("Unsupported float size: {}", bytes),
            },
            orco::ir::Type::Pointer(_) => self.object.target_config().pointer_type(),
            _ => todo!("type {:?}", r#type),
        }
    }
}

pub fn integer(bytes: std::num::NonZeroU16) -> cranelift_codegen::ir::Type {
    match bytes.get() {
        1 => cranelift_codegen::ir::types::I8,
        2 => cranelift_codegen::ir::types::I16,
        4 => cranelift_codegen::ir::types::I32,
        8 => cranelift_codegen::ir::types::I64,
        16 => cranelift_codegen::ir::types::I128,
        _ => unimplemented!("Unsupported integer size: {}", bytes),
    }
}
