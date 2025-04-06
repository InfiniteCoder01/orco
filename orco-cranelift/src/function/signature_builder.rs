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
            orco::Type::Int(bits) | orco::Type::Unsigned(bits) => cl::AbiParam::new(
                cl::Type::int(bits).unwrap_or_else(|| panic!("unsupported integer width {bits}")),
            ),
        }
    }
}
