use cranelift_module::Module;
use log::trace;

pub mod expression;
pub mod function;
pub mod types;

pub struct Object {
    pub object: cranelift_object::ObjectModule,
    pub functions: std::collections::HashMap<String, cranelift_module::FuncId>,
    pub constant_data: Option<(cranelift_module::DataId, Vec<u8>)>,
}

impl Object {
    pub fn new(isa: &str) -> Self {
        let flag_builder = cranelift_codegen::settings::builder();
        let isa_builder = cranelift_codegen::isa::lookup_by_name(isa).unwrap();
        let isa = isa_builder
            .finish(cranelift_codegen::settings::Flags::new(flag_builder))
            .unwrap();
        let object = cranelift_object::ObjectModule::new(
            cranelift_object::ObjectBuilder::new(
                isa,
                "foo",
                cranelift_module::default_libcall_names(),
            )
            .unwrap(),
        );

        Self {
            object,
            functions: std::collections::HashMap::new(),
            constant_data: None,
        }
    }
}

pub fn build(root: &orco::ir::Module) {
    trace!("Compiling module:\n{}", root);
    let mut object = Object::new("x86_64-unknown-linux-gnu");

    for (name, item) in &root.items {
        match item {
            orco::ir::Item::Function(function) => {
                object.declare_function(
                    name,
                    cranelift_module::Linkage::Export,
                    &function.signature,
                );
            }
            orco::ir::Item::ExternalFunction(signature) => {
                object.declare_function(name, cranelift_module::Linkage::Import, signature);
            }
        }
    }

    for (name, item) in &root.items {
        if let orco::ir::Item::Function(function) = item {
            object.build_function(root, name, function);
        }
    }

    if let Some((id, data)) = object.constant_data {
        object
            .object
            .define_data(
                id,
                &cranelift_module::DataDescription {
                    init: cranelift_module::Init::Bytes {
                        contents: data.as_slice().into(),
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
    }

    let object = object.object.finish();
    std::fs::write("foo.o", object.emit().unwrap()).unwrap();
}
