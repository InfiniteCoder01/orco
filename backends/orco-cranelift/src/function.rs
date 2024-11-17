use super::*;

impl Object {
    /// Declare a function in the object
    pub fn declare_function(&mut self, function: &dyn orco::symbol::Function) {
        trace!("Declaring function {}", function.name());
        let id = self
            .object
            .declare_function(
                function.name().as_ref(),
                cl::Linkage::Export,
                &self.convert_function_signature(function.signature()),
            )
            .unwrap();
        self.functions.insert(function.name().into_owned(), id);
    }

    /// Build the function's body
    pub fn build_function(&mut self, function: &dyn orco::symbol::Function) {
        info!("Compiling function {}", function.name());
        trace!("OrCo IR:\n{}", function as &dyn orco::symbol::Function);

        let id = *self
            .functions
            .get(function.name().as_ref())
            .expect("Function has to be declared before it is built!");

        let mut ctx = cl::codegen::Context::new();
        ctx.func = cl::codegen::ir::Function::with_name_signature(
            if cfg!(debug_assertions) {
                cl::codegen::ir::UserFuncName::testcase(function.name().as_ref())
            } else {
                cl::codegen::ir::UserFuncName::user(0, id.as_u32())
            },
            self.convert_function_signature(function.signature()),
        );

        {
            let mut function_ctx = cl::FunctionBuilderContext::new();
            let mut builder = cl::FunctionBuilder::new(&mut ctx.func, &mut function_ctx);
            let block = builder.create_block();
            builder.switch_to_block(block);
            builder.seal_block(block);
            builder.append_block_params_for_function_params(block);
            self.build_expression(&mut builder, function.body());
            builder.finalize();
        }
        self.object.define_function(id, &mut ctx).unwrap();
    }

    /// Convert OrCo function signature to Cranelift function signature
    pub fn convert_function_signature(
        &self,
        signature: &dyn orco::symbol::function::FunctionSignature,
    ) -> cl::Signature {
        cl::Signature {
            params: signature
                .parameters()
                .flat_map(|param| self.convert_type(param.r#type()).into_iter())
                .collect(),
            returns: self.convert_type(signature.return_type()),
            call_conv: cl::isa::CallConv::SystemV,
        }
    }
}
