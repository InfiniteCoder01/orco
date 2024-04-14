use cranelift_codegen::ir::*;
use cranelift_codegen::Context;
use cranelift_frontend::*;
use cranelift_module::Linkage;
use cranelift_module::Module;
use log::info;

pub fn build(module: &orco::ir::Module) {
    let flag_builder = cranelift_codegen::settings::builder();
    let isa_builder = cranelift_codegen::isa::lookup_by_name("x86_64-unknown-linux-gnu").unwrap();
    let isa = isa_builder
        .finish(cranelift_codegen::settings::Flags::new(flag_builder))
        .unwrap();
    let mut object = cranelift_object::ObjectModule::new(
        cranelift_object::ObjectBuilder::new(isa, "foo", cranelift_module::default_libcall_names())
            .unwrap(),
    );

    let puts_id = object
        .declare_function(
            "puts",
            cranelift_module::Linkage::Import,
            &Signature {
                params: vec![AbiParam::new(types::I64)],
                returns: vec![AbiParam::new(types::I32)],
                call_conv: cranelift_codegen::isa::CallConv::SystemV,
            },
        )
        .unwrap();

    let message_id = object
        .declare_data("message", cranelift_module::Linkage::Export, false, false)
        .unwrap();

    for (name, item) in &module.items {
        match item {
            orco::ir::Item::Function(function) => {
                info!("Compiling function {}", name);
                let sig = Signature {
                    params: vec![],
                    returns: vec![AbiParam::new(types::I32)],
                    call_conv: cranelift_codegen::isa::CallConv::SystemV,
                };

                let id = object
                    .declare_function(&name, Linkage::Export, &sig)
                    .unwrap();

                let mut ctx = Context::new();
                ctx.func = Function::with_name_signature(UserFuncName::user(0, id.as_u32()), sig);
                let mut func_ctx = FunctionBuilderContext::new();
                {
                    let mut bcx: FunctionBuilder =
                        FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
                    let block = bcx.create_block();
                    bcx.switch_to_block(block);
                    {
                        let local_id = object.declare_data_in_func(message_id, bcx.func);

                        let pointer = object.target_config().pointer_type();
                        let local_puts = object.declare_func_in_func(puts_id, bcx.func);
                        let msg_local = bcx.ins().symbol_value(pointer, local_id);
                        bcx.ins().call(local_puts, &[msg_local]);
                    }
                    let ret = bcx.ins().iconst(types::I32, 42);
                    bcx.ins().return_(&[ret]);
                }
                object.define_function(id, &mut ctx).unwrap();
            }
            _ => unimplemented!(),
        }
    }

    object
        .define_data(
            message_id,
            &cranelift_module::DataDescription {
                init: cranelift_module::Init::Bytes {
                    contents: (*b"Hello, world!\n\0").into(),
                },
                function_decls: Default::default(),
                data_decls: Default::default(),
                function_relocs: Default::default(),
                data_relocs: Default::default(),
                custom_segment_section: Default::default(),
                align: Default::default(),
            },
        )
        .unwrap();

    let mut object = object.finish();
    std::fs::write("foo.o", object.emit().unwrap()).unwrap();
}
