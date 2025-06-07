//! Cranelift backend for orco

use orco::backend as ob;

pub mod cl {
    pub use cranelift;
    pub use cranelift_module;
    pub use cranelift_object;

    pub use cranelift::prelude::*;
    pub use cranelift_module::{default_libcall_names, FuncId, Linkage, Module};
    pub use cranelift_object::{ObjectBuilder, ObjectModule, ObjectProduct};
}
