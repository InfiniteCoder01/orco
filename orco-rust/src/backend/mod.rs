pub mod cl {
    pub use cranelift::prelude::*;
    pub use cranelift_module::{FuncId, Module, default_libcall_names};
    pub use cranelift_object::{ObjectBuilder, ObjectModule};
}

use cl::Module;

pub mod function;
pub use function::FunctionBuilder;
use function::FunctionDecl;

pub struct Object {
    pub(crate) object: cl::ObjectModule,
    functions: std::collections::HashMap<crate::hir::FunctionId, FunctionDecl>,
}

impl Object {
    /// Create a new object from an OrCo IR and an ISA name
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
            object,
            functions: std::collections::HashMap::new(),
        }
    }

    pub fn declare_function(
        &mut self,
        handle: crate::hir::FunctionId,
        path: &crate::hir::Path,
        signature: &crate::hir::Signature,
    ) {
        let name = path.to_string();
        let signature = self.convert_signature(signature);
        let id = self
            .object
            .declare_function(&name, cranelift_module::Linkage::Export, &signature)
            .unwrap();
        self.functions.insert(
            handle,
            FunctionDecl {
                id,
                name,
                signature,
            },
        );
    }

    pub fn convert_type(&mut self, ty: &crate::hir::Type) -> Vec<cl::AbiParam> {
        match ty {
            crate::hir::Type::Path(path) => {
                panic!("Paths in backend are not allowed (found {})", path)
            }
            crate::hir::Type::Int(bits) => vec![cl::AbiParam::new(cl::Type::int(*bits).unwrap())],
            crate::hir::Type::Unsigned(bits) => {
                vec![cl::AbiParam::new(cl::Type::int(*bits).unwrap())]
            }
            crate::hir::Type::Tuple(items) => items
                .iter()
                .map(|ty| self.convert_type(ty))
                .flatten()
                .collect(),
        }
    }

    pub fn convert_signature(&mut self, signature: &crate::hir::Signature) -> cl::Signature {
        cl::Signature {
            params: signature
                .parameters
                .iter()
                .map(|param| self.convert_type(param))
                .flatten()
                .collect(),
            returns: self.convert_type(&signature.return_type),
            call_conv: cl::isa::CallConv::SystemV,
        }
    }

    pub fn build_function(
        &mut self,
        handle: crate::hir::FunctionId,
        build: impl FnOnce(&mut FunctionBuilder),
    ) {
        let decl = self.functions.get(&handle).unwrap();
        let mut ctx = cl::codegen::Context::new();
        ctx.func = cl::codegen::ir::Function::with_name_signature(
            if cfg!(debug_assertions) {
                cl::codegen::ir::UserFuncName::testcase(&decl.name)
            } else {
                cl::codegen::ir::UserFuncName::user(0, decl.id.as_u32())
            },
            decl.signature.clone(),
        );

        {
            let mut function_ctx = cl::FunctionBuilderContext::new();
            let mut builder =
                FunctionBuilder(cl::FunctionBuilder::new(&mut ctx.func, &mut function_ctx));
            let block = builder.0.create_block();
            builder.0.switch_to_block(block);
            builder.0.seal_block(block);
            builder.0.append_block_params_for_function_params(block);

            let print = self
                .object
                .declare_function(
                    "print",
                    cranelift_module::Linkage::Import,
                    &cl::Signature {
                        params: vec![cl::AbiParam::new(cl::types::I32)],
                        returns: vec![],
                        call_conv: cl::isa::CallConv::SystemV,
                    },
                )
                .unwrap();

            let func = self.object.declare_func_in_func(print, builder.0.func);
            let value = builder.0.ins().iconst(cl::types::I32, 42);
            use cl::InstBuilder;
            builder.0.ins().call(func, &[value]);

            build(&mut builder);
            builder.0.finalize();
        }

        self.object.define_function(decl.id, &mut ctx).unwrap();
    }
}
