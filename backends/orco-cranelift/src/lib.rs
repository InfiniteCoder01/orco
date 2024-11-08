// #![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use cranelift::prelude::*;
use cranelift_module::Module;
use log::*;

/// Build expressions
pub mod expression;
/// Declare and build functions
pub mod function;
// /// Declare and build modules
// pub mod module;
// /// Declare and convert types
// pub mod types;

/// Object, translation unit, a wrapper around `cranelift_object::ObjectModule`
pub struct Object {
    /// Cranelift object
    pub object: cranelift_object::ObjectModule,
    /// Functions table
    pub functions: std::collections::HashMap<String, cranelift_module::FuncId>,
    /// Constant pool
    pub constant_data: Option<(cranelift_module::DataId, Vec<u8>)>,
}

impl Object {
    /// Create a new object from an OrCo IR and an ISA name
    pub fn new(isa: &str) -> Self {
        let flag_builder = settings::builder();
        let isa_builder = isa::lookup_by_name(isa).unwrap();
        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
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
