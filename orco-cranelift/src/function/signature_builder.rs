use super::{FunctionDecl, Object, cl, ob};
use cl::Module;

pub(crate) struct SignatureBuilder<'a> {
    object: &'a mut Object,
    id: ob::FunctionId,
    name: String,
    signature: cl::Signature,
}

impl<'a> SignatureBuilder<'a> {
    pub(crate) fn new(object: &'a mut Object, id: ob::FunctionId, name: String) -> Self {
        Self {
            object,
            id,
            name,
            signature: cl::Signature {
                params: Vec::new(),
                returns: Vec::new(),
                call_conv: cl::isa::CallConv::Fast,
            },
        }
    }
}

impl ob::SignatureBuilder for SignatureBuilder<'_> {
    fn finish(self: Box<Self>) {
        let cl_id = self
            .object
            .object
            .lock()
            .unwrap()
            .declare_function(
                &self.name,
                cranelift_module::Linkage::Export,
                &self.signature,
            )
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
