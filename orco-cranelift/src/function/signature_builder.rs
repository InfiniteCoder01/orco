use super::{FunctionDecl, Object, cl, ob};
use cl::Module;

pub(crate) struct SignatureBuilder<'a> {
    object: &'a mut Object,
    id: ob::FunctionId,
    name: String,
    signature: cl::Signature,
    linkage: cl::Linkage,
}

impl<'a> SignatureBuilder<'a> {
    pub(crate) fn new(object: &'a mut Object, id: ob::FunctionId, name: String) -> Self {
        let sig = object.registry.get_fn(id);
        let signature = cl::Signature {
            params: sig
                .params
                .iter()
                .map(|param| object.convert_type(param.ty))
                .collect(),
            returns: Vec::new(),
            call_conv: cl::isa::CallConv::Fast,
        };
        Self {
            object,
            id,
            name,
            signature,
            linkage: cl::Linkage::Local,
        }
    }
}

impl ob::SignatureBuilder for SignatureBuilder<'_> {
    fn public(&mut self) {
        self.linkage = cl::Linkage::Export;
    }

    fn external(&mut self) {
        self.linkage = cl::Linkage::Import;
    }

    fn finish(self: Box<Self>) {
        let cl_id = self
            .object
            .object
            .lock()
            .unwrap()
            .declare_function(&self.name, self.linkage, &self.signature)
            .unwrap();
        self.object.functions.insert(
            self.id,
            FunctionDecl {
                cl_id,
                name: self.name,
                signature: self.signature,
            },
        );
    }
}

impl Object {
    pub(crate) fn convert_type(&self, ty: orco::Type) -> cl::AbiParam {
        match ty {
            orco::Type::Int(8) | orco::Type::Unsigned(8) => cl::AbiParam::new(cl::types::I8),
            orco::Type::Int(16) | orco::Type::Unsigned(16) => cl::AbiParam::new(cl::types::I16),
            orco::Type::Int(32) | orco::Type::Unsigned(32) => cl::AbiParam::new(cl::types::I32),
            orco::Type::Int(64) | orco::Type::Unsigned(64) => cl::AbiParam::new(cl::types::I64),
            orco::Type::Int(128) | orco::Type::Unsigned(128) => cl::AbiParam::new(cl::types::I128),
            orco::Type::Int(bits) => panic!("unsupported integer width {bits}"),
            orco::Type::Unsigned(bits) => panic!("unsupported unsigned integer width {bits}"),
        }
    }
}
