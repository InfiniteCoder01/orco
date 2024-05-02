use cranelift_module::Module;

impl crate::Object<'_> {
    pub fn convert_type(&self, r#type: &orco::ir::Type) -> cranelift_codegen::ir::Type {
        match r#type {
            orco::ir::Type::Int(bytes) | orco::ir::Type::Unsigned(bytes) => integer(*bytes),
            orco::ir::Type::Float(bytes) => match bytes.get() {
                4 => cranelift_codegen::ir::types::F32,
                8 => cranelift_codegen::ir::types::F64,
                _ => unimplemented!("Unsupported float size: {}", bytes),
            },
            orco::ir::Type::Bool => cranelift_codegen::ir::types::I8,
            orco::ir::Type::Char => cranelift_codegen::ir::types::I8,

            orco::ir::Type::Pointer(_) => self.object.target_config().pointer_type(),
            orco::ir::Type::Custom(_) => todo!(),

            orco::ir::Type::Never => panic!("Can't convert a never type"),
            orco::ir::Type::Unit => panic!("Can't convert a unit type"),

            orco::ir::Type::Wildcard => panic!("Type inference wasn't done properly"),
            orco::ir::Type::IntegerWildcard => panic!("Type inference wasn't done properly"),
            orco::ir::Type::TypeVariable(_) => panic!("Type inference wasn't done properly"),
            orco::ir::Type::Error => panic!("IR contains errors!"),
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
