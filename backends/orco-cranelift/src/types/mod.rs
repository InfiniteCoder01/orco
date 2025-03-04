use crate::cl;

impl crate::Object {
    pub fn convert_type(&self, ty: &orco::Type) -> Vec<cl::AbiParam> {
        match ty {
            orco::Type::Wildcard => {
                panic!("Wildcard type encountered on backend phase. Type inference was most likely not done correctly")
            }
            orco::Type::Never => Vec::new(),
            orco::Type::Unit => Vec::new(),
                    128 => cl::types::I128,
                    _ => cl::types::INVALID,
                })]
            }
            orco::Type::Float(bits) => vec![cl::AbiParam::new(match bits {
                16 => cl::types::F16,
                32 => cl::types::F32,
                64 => cl::types::F64,
                128 => cl::types::F128,
                _ => cl::types::INVALID,
            })],
            orco::Type::Fn(function_signature) => todo!(),
            orco::Type::Unresolved(_) => todo!(),
        }
    }

    /// Convert OrCo function signature to Cranelift function signature
    pub fn convert_function_signature(
        &self,
        signature: &orco::types::FunctionSignature,
    ) -> cl::Signature {
        cl::Signature {
            params: signature
                .parameters
                .iter()
                .flat_map(|(name, ty)| self.convert_type(ty).into_iter())
                .collect(),
            returns: self.convert_type(signature.return_type.as_ref()),
            call_conv: cl::isa::CallConv::SystemV,
        }
    }
}
