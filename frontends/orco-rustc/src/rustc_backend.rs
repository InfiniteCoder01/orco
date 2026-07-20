// This prevents duplicating functions and statics that are already part of the host rustc process.
#[allow(unused_extern_crates)]
extern crate rustc_driver;

use std::any::Any;

use rustc_middle::ty::TyCtxt;
use rustc_session::Session;

/// `rustc_ssa_codegen` backend for orco
pub struct OrcoCodegenBackend;

impl rustc_codegen_ssa::traits::CodegenBackend for OrcoCodegenBackend {
    fn name(&self) -> &'static str {
        "orco codegen"
    }

    fn target_cpu(&self, _sess: &Session) -> String {
        "cc".to_owned()
    }

    fn codegen_crate(&self, tcx: TyCtxt<'_>) -> Box<dyn Any> {
        tracing::info!("Name: {}", tcx.crate_name(rustc_hir::def_id::LOCAL_CRATE));
        // rustc_middle::mir::write_mir_pretty(tcx, &mut std::io::stdout()).unwrap();
        let items = tcx.hir_crate_items(());

        let ir = orco_ir::Store::new();
        crate::declare(tcx, &ir, items);
        // crate::codegen(tcx, &ir, items);

        let backend = orco_cgen::Backend::new();
        ir.declare_mono(&backend);
        print!("{backend}");

        std::process::exit(0)
    }

    fn join_codegen(
        &self,
        _ongoing_codegen: Box<dyn Any>,
        _sess: &Session,
        _outputs: &rustc_session::config::OutputFilenames,
        _crate_info: &rustc_codegen_ssa::CrateInfo,
    ) -> (
        rustc_codegen_ssa::CompiledModules,
        rustc_middle::dep_graph::WorkProductMap,
    ) {
        (
            rustc_codegen_ssa::CompiledModules {
                modules: Vec::new(),
                allocator_module: None,
            },
            Default::default(),
        )
    }
}
