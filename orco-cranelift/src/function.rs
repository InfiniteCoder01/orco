use super::{Object, cl, ob};
use cl::Module;

pub(crate) struct FunctionDecl {
    pub(crate) cl_id: cl::FuncId,
    pub(crate) name: String,
    pub(crate) signature: cl::Signature,
}

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

struct FunctionBuilderInner {
    ctx: cl::codegen::Context,
    fctx: cl::FunctionBuilderContext,
}

pub(crate) struct FunctionBuilder<'a> {
    object: &'a Object,
    id: ob::FunctionId,
    inner: std::pin::Pin<Box<FunctionBuilderInner>>,
    builder: cl::FunctionBuilder<'a>,
}

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn new(object: &'a Object, id: ob::FunctionId) -> Self {
        let decl = object
            .functions
            .get(&id)
            .expect("Trying to build an undeclared function");

        let mut ctx = cl::codegen::Context::new();
        ctx.func = cl::codegen::ir::Function::with_name_signature(
            if cfg!(debug_assertions) {
                cl::codegen::ir::UserFuncName::testcase(&decl.name)
            } else {
                cl::codegen::ir::UserFuncName::user(0, id.0 as _)
            },
            decl.signature.clone(),
        );

        let mut inner = Box::pin(FunctionBuilderInner {
            ctx,
            fctx: cl::FunctionBuilderContext::new(),
        });

        let builder =
            cl::FunctionBuilder::new(unsafe { &mut *(&mut inner.ctx.func as *mut _) }, unsafe {
                &mut *(&mut inner.fctx as *mut _)
            });

        Self {
            object,
            id,
            inner,
            builder,
        }
    }
}

impl ob::FunctionBuilder for FunctionBuilder<'_> {
    fn unit(&mut self) -> ob::function::SSAValue {
        todo!()
    }

    fn i32(&mut self, value: i32) -> ob::function::SSAValue {
        todo!()
    }

    fn ret(&mut self, value: ob::function::SSAValue) {
        todo!()
    }

    fn finish(mut self: Box<Self>) {
        let decl = self
            .object
            .functions
            .get(&self.id)
            .expect("Trying to build an undeclared function");

        self.builder.finalize();
        self.object
            .object
            .lock()
            .unwrap()
            .define_function(decl.cl_id, &mut self.inner.ctx)
            .unwrap();
    }
}
