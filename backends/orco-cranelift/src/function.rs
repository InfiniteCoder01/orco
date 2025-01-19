use super::*;

impl Object {
    /// Declare a function in the object
    pub fn declare_function(&mut self, name: &str, function: &orco::expression::Function) {
        trace!("Declaring function {:?}", name);
        let id = self
            .object
            .declare_function(
                name,
                cl::Linkage::Export,
                &self.convert_function_signature(&function.signature),
            )
            .unwrap();
        self.functions.insert(name.to_owned(), id);
    }

    /// Build the function
    pub fn build_function(&mut self, name: &str, function: &orco::expression::Function) {
        info!("Compiling function {:?}", name);
        trace!("OrCo IR:\n{}", function);

        let id = *self
            .functions
            .get(name)
            .expect("Function has to be declared before it is built!");

        let mut ctx = cl::codegen::Context::new();
        ctx.func = cl::codegen::ir::Function::with_name_signature(
            if cfg!(debug_assertions) {
                cl::codegen::ir::UserFuncName::testcase(function.name().unwrap_or(name))
            } else {
                cl::codegen::ir::UserFuncName::user(0, id.as_u32())
            },
            self.convert_function_signature(&function.signature),
        );

        {
            let mut function_ctx = cl::FunctionBuilderContext::new();
            let mut builder = cl::FunctionBuilder::new(&mut ctx.func, &mut function_ctx);
            let block = builder.create_block();
            builder.switch_to_block(block);
            builder.seal_block(block);
            builder.append_block_params_for_function_params(block);
            self.build_function_body(&mut builder, function);
            builder.finalize();
        }
        self.object.define_function(id, &mut ctx).unwrap();
    }

    /// Build the function's body using a function builder
    pub fn build_function_body(
        &mut self,
        builder: &mut cranelift::prelude::FunctionBuilder,
        function: &orco::expression::Function,
    ) {
        match &function.body {
            orco::expression::function::FunctionBody::Block(_, body) => {
                for expr in body {
                    self.build_expression(builder, expr);
                }
            }
            orco::expression::function::FunctionBody::External(_) => todo!(),
        }
    }
}
