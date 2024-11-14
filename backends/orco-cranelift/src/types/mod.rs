use super::*;

impl Object {
    pub fn convert_type(&self, ty: orco::Type) -> Vec<cl::AbiParam> {
        match ty{
            orco::Type::Wildcard => panic!("Wildcard type encountered on backend phase. Type inference was most likely not done correctly"),
            orco::Type::Never => Vec::new(),
            orco::Type::Unit => Vec::new(),
            orco::Type::Integer(bits) | orco::Type::Unsigned(bits) => vec![cl::AbiParam::new(match bits {
                8 => cl::types::I8,
                16 => cl::types::I16,
                32 => cl::types::I32,
                64 => cl::types::I64,
                128 => cl::types::I128,
                _ => cl::types::INVALID,
            })],
            orco::Type::Float(bits) => vec![cl::AbiParam::new(match bits {
                16 => cl::types::F16,
                32 => cl::types::F32,
                64 => cl::types::F64,
                128 => cl::types::F128,
                _ => cl::types::INVALID,
            })],
        }
    }
}
