use super::*;
use cranelift_codegen::entity::EntityRef;
use cranelift_codegen::ir::*;
use cranelift_codegen::Context;
use cranelift_frontend::*;
use cranelift_module::Module;
use log::{info, trace};

impl crate::Object<'_> {
    /// Declare a function in the object
    pub fn declare_function(
        &mut self,
        name: Path,
        linkage: cranelift_module::Linkage,
        signature: &orco::ir::expression::function::Signature,
    ) -> cranelift_module::FuncId {
        trace!("Declaring function {}", name);
        let sig = self.convert_function_signature(signature);
        let id = self
            .object
            .declare_function(&name.to_string(), linkage, &sig)
            .unwrap();
        self.functions.insert(name, id);
        id
    }

    /// Build a function in the object, must declare it first with [`Self::declare_function`]
    pub fn build_function(
        &mut self,
        name: &Path,
        function: &orco::ir::expression::function::Function,
    ) {
        info!("Compiling function {}", name);
        trace!("OrCo IR:\n{}", function);

        let id = *self.functions.get(name).expect("Function wasn't declared");
        let sig = self.convert_function_signature(&function.signature);
        let mut ctx = Context::new();
        ctx.func = Function::with_name_signature(
            if cfg!(debug_assertions) {
                UserFuncName::testcase(name.to_string())
            } else {
                UserFuncName::user(0, id.as_u32())
            },
            sig,
        );
        {
            let mut function_ctx = FunctionBuilderContext::new();
            let mut builder = FunctionBuilder::new(&mut ctx.func, &mut function_ctx);
            let block = builder.create_block();
            builder.append_block_params_for_function_params(block);
            builder.switch_to_block(block);
            builder.seal_block(block);
            for (arg, value) in std::iter::zip(
                &function.signature.args.inner,
                builder.block_params(block).to_vec(),
            ) {
                let variable = Variable::new(*arg.id.try_lock().unwrap() as _);
                builder.declare_var(variable, self.convert_type(&arg.r#type.try_lock().unwrap()));
                builder.def_var(variable, value);
            }
            let body = function.body.try_lock().unwrap();
            let return_value = self.build_expression(&mut builder, &body);
            if body.get_type() != orco::ir::Type::Never {
                builder
                    .ins()
                    .return_(&return_value.into_iter().collect::<Vec<_>>());
            }
            builder.finalize();
        }
        self.object.define_function(id, &mut ctx).unwrap();
    }

    /// Convert OrCo function signature to Cranelift function signature
    pub fn convert_function_signature(
        &self,
        signature: &orco::ir::expression::function::Signature,
    ) -> cranelift_codegen::ir::Signature {
        use cranelift_codegen::ir::AbiParam;
        cranelift_codegen::ir::Signature {
            params: signature
                .args
                .iter()
                .map(|arg| AbiParam::new(self.convert_type(&arg.r#type.try_lock().unwrap())))
                .collect(),
            returns: if *signature.return_type == orco::ir::Type::Unit {
                vec![]
            } else {
                vec![AbiParam::new(self.convert_type(&signature.return_type))]
            },
            call_conv: cranelift_codegen::isa::CallConv::SystemV,
        }
    }
}
