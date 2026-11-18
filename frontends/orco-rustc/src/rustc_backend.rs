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
        rustc_middle::mir::write_mir_pretty(tcx, &mut std::io::stdout()).unwrap();

        let mut module = orco::Module::new();
        module.symbols.insert(
            2.into(),
            orco::Symbol::Function(orco::Function {
                name: Some("core::mem::drop".to_owned()),
                generics: vec![orco::TyParam {
                    name: Some("T".to_owned()),
                }],
                params: vec![(None, orco::Type::Param(0))],
                return_type: orco::Type::Unit,
                attrs: Default::default(),
                body: None,
            }),
        );

        rustc_public::rustc_internal::run(tcx, || {
            crate::declare(rustc_public::local_crate(), &mut module);
            crate::codegen(rustc_public::local_crate(), &mut module);
        })
        .unwrap();

        module.monomorphize();
        module.name_anonymous_structs();
        print!("{module}");
        print!("{}", orco_cgen::FmtModule(&module));

        std::process::exit(0)
    }

    fn join_codegen(
        &self,
        _ongoing_codegen: Box<dyn Any>,
        _sess: &Session,
        _incr_comp_session: Option<&rustc_session::IncrCompSession>,
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
