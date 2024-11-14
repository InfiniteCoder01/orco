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
                &cl::Signature {
                    params: Vec::new(),
                    returns: self.convert_type(function.return_type()),
                    call_conv: cl::isa::CallConv::SystemV,
                },
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
            cl::Signature {
                params: Vec::new(),
                returns: self.convert_type(function.return_type()),
                call_conv: cl::isa::CallConv::SystemV,
            },
        );

        {
            let mut function_ctx = cl::FunctionBuilderContext::new();
            let mut builder = cl::FunctionBuilder::new(&mut ctx.func, &mut function_ctx);
            let block = builder.create_block();
            builder.switch_to_block(block);
            builder.seal_block(block);
            self.build_expression(&mut builder, function.body());
            builder.finalize();
        }
        self.object.define_function(id, &mut ctx).unwrap();
    }

    // /// Convert OrCo function signature to Cranelift function signature
    // pub fn convert_function_signature(
    //     &self,
    //     signature: &orco::ir::expression::function::Signature,
    // ) -> cranelift_codegen::ir::Signature {
    //     use cranelift_codegen::ir::AbiParam;
    //     cranelift_codegen::ir::Signature {
    //         params: signature
    //             .args
    //             .iter()
    //             .map(|arg| AbiParam::new(self.convert_type(&arg.r#type.try_lock().unwrap())))
    //             .collect(),
    //         returns: if *signature.return_type == orco::ir::Type::Unit {
    //             vec![]
    //         } else {
    //             vec![AbiParam::new(self.convert_type(&signature.return_type))]
    //         },
    //         call_conv: cranelift_codegen::isa::CallConv::SystemV,
    //     }
    // }
}
