#![doc = include_str!("../README.md")]
use cranelift_module::Module;
use log::*;

mod cl {
    pub(super) use cranelift::prelude::*;
    pub(super) use cranelift_module::*;
    pub(super) use cranelift_object::*;
}

pub mod expression;
/// Declare and build functions
pub mod function;
/// Declare and convert types
pub mod types;

/// Object, translation unit, a wrapper around [`cl::ObjectModule`]
pub struct Object {
    /// Cranelift object
    pub object: cl::ObjectModule,
    /// Functions table
    pub functions: std::collections::HashMap<String, cl::FuncId>,
    /// Constant pool
    pub constant_data: Option<(cl::DataId, Vec<u8>)>,
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
            constant_data: None,
        }
    }

    fn declare_module(&mut self, unit: &dyn orco::Unit) {
        for symbol in unit.symbols() {
            match symbol {
                orco::Symbol::Function(function) => {
                    self.declare_function(&*function.try_read().unwrap())
                }
            }
        }
    }

    fn build_module(&mut self, unit: &dyn orco::Unit) {
        for symbol in unit.symbols() {
            match symbol {
                orco::Symbol::Function(function) => {
                    self.build_function(&*function.try_read().unwrap())
                }
            }
        }
    }
}

/// Build OrCo IR Unit
pub fn build(unit: &dyn orco::Unit) {
    debug!("Compiling module:\n{}", unit as &dyn orco::Unit);
    let mut object = Object::new("x86_64-unknown-linux-gnu");
    object.declare_module(unit);
    object.build_module(unit);

    if let Some((id, data)) = object.constant_data {
        object
            .object
            .define_data(
                id,
                &cl::DataDescription {
                    init: cl::Init::Bytes {
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
