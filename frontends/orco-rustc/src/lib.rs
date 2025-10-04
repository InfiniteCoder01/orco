//! rustc frontend for orco
#![feature(rustc_private)]
#![warn(missing_docs)]

extern crate rustc_ast;
extern crate rustc_codegen_ssa;
extern crate rustc_const_eval;
extern crate rustc_data_structures;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_metadata;
extern crate rustc_middle;
extern crate rustc_public;
extern crate rustc_session;
extern crate rustc_span;
extern crate tracing;

// This prevents duplicating functions and statics that are already part of the host rustc process.
#[allow(unused_extern_crates)]
extern crate rustc_driver;

/// Code generation is used to define functions and other items
pub mod codegen;
/// Declaration is used to declare functions and other items,
/// useful for generating bindings
pub mod declare;

use std::any::Any;

use rustc_codegen_ssa::traits::CodegenBackend;
use rustc_middle::ty::TyCtxt;
use rustc_session::Session;

/// rustc_ssa_codegen backend for orco
pub struct OrcoCodegenBackend;

impl CodegenBackend for OrcoCodegenBackend {
    fn locale_resource(&self) -> &'static str {
        // FIXME(rust-lang/rust#100717) - cranelift codegen backend is not yet translated
        ""
    }

    fn name(&self) -> &'static str {
        "orco codegen"
    }

    fn codegen_crate(&self, tcx: TyCtxt<'_>) -> Box<dyn Any> {
        tracing::info!("Name: {}", tcx.crate_name(rustc_hir::def_id::LOCAL_CRATE));
        let items = tcx.hir_crate_items(());
        let mut backend = orco_cgen::Backend::new();
        declare::declare(tcx, &mut backend, items);
        codegen::define(tcx, &mut backend, items);
        println!("{backend}");
        std::process::exit(0)
    }

    fn join_codegen(
        &self,
        ongoing_codegen: Box<dyn Any>,
        _sess: &Session,
        _outputs: &rustc_session::config::OutputFilenames,
    ) -> (
        rustc_codegen_ssa::CodegenResults,
        rustc_data_structures::fx::FxIndexMap<
            rustc_middle::dep_graph::WorkProductId,
            rustc_middle::dep_graph::WorkProduct,
        >,
    ) {
        (
            rustc_codegen_ssa::CodegenResults {
                modules: Vec::new(),
                allocator_module: None,
                crate_info: *ongoing_codegen.downcast().unwrap(),
            },
            rustc_data_structures::fx::FxIndexMap::default(),
        )
    }
}

/// This is the entrypoint for a hot plugged rustc_codegen_orco
#[unsafe(no_mangle)]
pub fn __rustc_codegen_backend() -> Box<dyn CodegenBackend> {
    Box::new(OrcoCodegenBackend)
}
