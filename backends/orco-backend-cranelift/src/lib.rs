#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use cranelift_module::Module;
use log::debug;
use orco::Path;

/// Build expressions
pub mod expression;
/// Declare and build functions
pub mod function;
/// Declare and build modules
pub mod module;
/// Declare and convert types
pub mod types;

/// Object, translation unit, a wrapper around `cranelift_object::ObjectModule`
pub struct Object {
    /// Cranelift object
    pub object: cranelift_object::ObjectModule,
    /// Functions table
    pub functions: std::collections::HashMap<Path, cranelift_module::FuncId>,
    /// Constant pool
    pub constant_data: Option<(cranelift_module::DataId, Vec<u8>)>,
}

impl Object {
    /// Create a new object from an OrCo IR and an ISA name
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

/// Build the OrCo IR module
pub fn build(root: &orco::ir::Module) {
    debug!("Compiling module:\n{}", root);
    let mut object = Object::new("x86_64-unknown-linux-gnu");
    object.build_module(root, &Path::new());

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
