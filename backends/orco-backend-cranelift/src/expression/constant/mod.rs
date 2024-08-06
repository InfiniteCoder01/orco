use super::*;

/// Constant pool
pub mod pool;

impl crate::Object<'_> {
    /// Build a constant expression
    pub fn build_constant(
        &mut self,
        builder: &mut FunctionBuilder,
        value: &orco::ir::expression::Constant,
    ) -> Option<Value> {
        use orco::ir::expression::Constant;
        use orco::ir::Type;
        match value {
            Constant::Integer { value, r#type, ..} => {
                if let Type::Int(size) = r#type {
                    if size.get() > 8 {
                        todo!("Cranelift integer constants bigger than 64 bits");
                    } else if !size.get().is_power_of_two(){
                        panic!("Invalid or unsupported integer constant type {}! Did you run type checking/inference?", r#type);
                    }
                } else {
                    panic!("Invalid or unsupported integer constant type {}! Did you run type checking/inference?", r#type);
                }
                Some(
                    builder
                        .ins()
                        .iconst(self.convert_type(r#type), *value as i64),
                )
            }
            Constant::Float { value, r#type, .. } => match r#type {
                Type::Float(size) if size.get() == 4 => Some(builder.ins().f32const(*value as f32)),
                Type::Float(size) if size.get() == 8 => Some(builder.ins().f64const(*value)),
                _ => panic!("Invalid or unsupported float constant type {}! Did you run type checking/inference?", r#type),
            },
            Constant::CString(bytes, ..) => {
                Some(self.add_constant_to_pool(builder, bytes))
            }
        }
    }

    /// Build a constant value
    pub fn build_constant_value(
        &mut self,
        builder: &mut FunctionBuilder,
        value: &orco::interpreter::Value,
        r#type: &orco::ir::Type,
    ) -> Option<Value> {
        use orco::ir::Type;
        match r#type {
            Type::Int(size) => {
                Some(
                    builder
                        .ins()
                        .iconst(self.convert_type(r#type), match size.get(){
                            1 => *value.as_ref::<i8>() as i64,
                            2 => *value.as_ref::<i16>() as i64,
                            4 => *value.as_ref::<i32>() as i64,
                            8 => *value.as_ref::<i64>(),
                            16 => todo!("Cranelift integer constants bigger than 64 bits"),
                            _ => panic!("Invalid or unsupported integer constant value type {}! Did you run type checking/inference?", r#type),
                        }),
                )
            }
            Type::Unsigned(size) => {
                Some(
                    builder
                        .ins()
                        .iconst(self.convert_type(r#type), match size.get(){
                            1 => *value.as_ref::<u8>() as i64,
                            2 => *value.as_ref::<u16>() as i64,
                            4 => *value.as_ref::<u32>() as i64,
                            8 => *value.as_ref::<u64>() as i64,
                            16 => todo!("Cranelift integer constants bigger than 64 bits"),
                            _ => panic!("Invalid or unsupported unsigned integer constant value type {}! Did you run type checking/inference?", r#type),
                        }),
                )
            }
            Type::Float(size) => {
                match size.get() {
                    4 => Some(builder.ins().f32const(*value.as_ref::<f32>())),
                    8 => Some(builder.ins().f64const(*value.as_ref::<f64>())),
                    _ => panic!("Invalid or unsupported flaot constant value type {}! Did you run type checking/inference?", r#type),
                }
            }
            unsupported => panic!("Unsupported constant value type {}!", unsupported),
        }
    }
}
