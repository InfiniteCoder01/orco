// This prevents duplicating functions and statics that are already part of the host rustc process.
#[allow(unused_extern_crates)]
extern crate rustc_driver;

use std::any::Any;

use rustc_middle::ty::TyCtxt;
use rustc_session::Session;

/// rustc_ssa_codegen backend for orco
pub struct OrcoCodegenBackend;

impl rustc_codegen_ssa::traits::CodegenBackend for OrcoCodegenBackend {
    fn locale_resource(&self) -> &'static str {
        // FIXME(rust-lang/rust#100717) - orco codegen backend is not yet translated
        ""
    }

    fn name(&self) -> &'static str {
        "orco codegen"
    }

    fn codegen_crate(&self, tcx: TyCtxt<'_>) -> Box<dyn Any> {
        tracing::info!("Name: {}", tcx.crate_name(rustc_hir::def_id::LOCAL_CRATE));
        let items = tcx.hir_crate_items(());
        let backend = orco_cgen::Backend::new();
        crate::declare(tcx, &backend, items);
        crate::codegen(tcx, &backend, items);
        println!("{}", backend);
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
