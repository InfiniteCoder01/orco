use cranelift_codegen::{ir::*, Context};
use cranelift_frontend::*;
use cranelift_module::Module;
use log::{info, trace};

impl crate::Object {
    pub fn declare_function(
        &mut self,
        name: &str,
        linkage: cranelift_module::Linkage,
        signature: &orco::ir::item::function::Signature,
    ) -> cranelift_module::FuncId {
        trace!("Declaring function {}", name);
        let sig = convert_signature(signature);
        let id = self.object.declare_function(name, linkage, &sig).unwrap();
        self.functions.insert(name.to_string(), id);
        id
    }

    pub fn build_function(&mut self, name: &str, function: &orco::ir::item::function::Function) {
        info!("Compiling function {}", name);
        trace!("OrCo IR:\n{}", function);

        let id = *self.functions.get(name).expect("Function wasn't declared");
        let sig = convert_signature(&function.signature);
        let mut ctx = Context::new();
        ctx.func = Function::with_name_signature(UserFuncName::user(0, id.as_u32()), sig);
        {
            let mut function_ctx = FunctionBuilderContext::new();
            let mut builder = FunctionBuilder::new(&mut ctx.func, &mut function_ctx);
            let block = builder.create_block();
            builder.switch_to_block(block);
            self.build_block(&mut builder, &function.body);
        }
        self.object.define_function(id, &mut ctx).unwrap();
    }
}

pub fn convert_signature(
    signature: &orco::ir::item::function::Signature,
) -> cranelift_codegen::ir::Signature {
    use cranelift_codegen::ir::AbiParam;
    cranelift_codegen::ir::Signature {
        params: vec![],
        returns: if signature.return_type == orco::ir::Type::Unit {
            vec![]
        } else {
            vec![AbiParam::new(crate::types::convert(&signature.return_type))]
        },
        call_conv: cranelift_codegen::isa::CallConv::SystemV,
    }
}
