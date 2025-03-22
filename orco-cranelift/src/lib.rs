use orco::backend as ob;

pub mod cl {
    pub use cranelift;
    pub use cranelift_module;
    pub use cranelift_object;

    pub use cranelift::prelude::*;
    pub use cranelift_module::{FuncId, Module, default_libcall_names};
    pub use cranelift_object::{ObjectBuilder, ObjectModule, ObjectProduct};
}

mod function;
use function::{FunctionBuilder, FunctionDecl, SignatureBuilder};

pub struct Object {
    pub(crate) object: std::sync::Mutex<cl::ObjectModule>,
    pub(crate) functions: std::collections::HashMap<ob::FunctionId, FunctionDecl>,
}

impl Object {
    /// Create a new object from an ISA name
    pub fn new(isa: &str) -> Self {
        let flag_builder = cl::settings::builder();
        let isa_builder = cl::isa::lookup_by_name(isa).unwrap();
        let isa = isa_builder
            .finish(cl::settings::Flags::new(flag_builder))
            .unwrap();
        let object = cl::ObjectModule::new(
            cl::ObjectBuilder::new(isa, "foo", cl::default_libcall_names()).unwrap(),
        );

        Self {
            object: object.into(),
            functions: std::collections::HashMap::new(),
        }
    }

    pub fn finish(self) -> cl::ObjectProduct {
        self.object.into_inner().unwrap().finish()
    }
}

impl ob::Backend for Object {
    fn declare_function(
        &mut self,
        id: ob::FunctionId,
        name: impl ToString,
    ) -> Box<dyn ob::SignatureBuilder + '_> {
        Box::new(SignatureBuilder::new(self, id, name.to_string()))
    }

    fn build_function(&self, id: ob::FunctionId) -> Box<dyn ob::FunctionBuilder + '_> {
        Box::new(FunctionBuilder::new(self, id))
    }
}
