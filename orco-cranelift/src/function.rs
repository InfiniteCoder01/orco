use super::{Object, cl, ob};
use cl::Module;
use cranelift::prelude::InstBuilder;

mod signature_builder;
pub(crate) use signature_builder::SignatureBuilder;

pub(crate) struct FunctionDecl {
    pub(crate) cl_id: cl::FuncId,
    pub(crate) name: String,
    pub(crate) signature: cl::Signature,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum SSAValue {
    Split(Vec<cl::Value>),
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
    ssa: Vec<SSAValue>,
}

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn new(object: &'a Object, id: ob::FunctionId) -> Self {
        let decl = object
            .functions
            .get(&id)
            .expect("trying to build an undeclared function");

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

        let mut builder =
            cl::FunctionBuilder::new(unsafe { &mut *(&mut inner.ctx.func as *mut _) }, unsafe {
                &mut *(&mut inner.fctx as *mut _)
            });

        let entry = builder.create_block();
        builder.switch_to_block(entry);
        builder.append_block_params_for_function_params(entry);
        builder.seal_block(entry);

        Self {
            object,
            id,
            inner,
            builder,
            ssa: vec![SSAValue::Split(Vec::new())],
        }
    }

    fn alloc_ssa(&mut self, value: SSAValue) -> ob::SSAValue {
        let id = self.ssa.len();
        self.ssa.push(value);
        ob::SSAValue(id)
    }
}

impl ob::FunctionBuilder for FunctionBuilder<'_> {
    fn unit(&mut self) -> ob::SSAValue {
        ob::SSAValue(0)
    }

    fn i32(&mut self, value: i32) -> ob::SSAValue {
        todo!()
    }

    fn ret(&mut self, value: ob::SSAValue) {
        self.builder.ins().return_(
            match self.ssa.get(value.0).expect("got an invalid SSA id") {
                SSAValue::Split(values) => values,
            },
        );
    }

    fn finish(mut self: Box<Self>) {
        let decl = self
            .object
            .functions
            .get(&self.id)
            .expect("trying to build an undeclared function");

        self.builder.finalize();
        self.object
            .object
            .lock()
            .unwrap()
            .define_function(decl.cl_id, &mut self.inner.ctx)
            .unwrap();
    }
}
