//! rust-analyzer powered rust frontend for orco

pub mod ra {
    pub use ra_ap_base_db::{CrateOrigin, CrateWorkspaceData};
    pub use ra_ap_hir as hir;
    pub use ra_ap_hir_def as def;
    pub use ra_ap_hir_ty as ty;
    pub use ra_ap_ide as ide;
    pub use ra_ap_vfs as vfs;
}

use orco::backend as ob;
use orco::frontend as of;
use ra_ap_hir::db::HirDatabase;

pub mod types;

/// rust-analyzer frontend, uses multiple [Sources], one per crate.
#[derive(Debug, Default)]
pub struct RAFrontend {
    pub analysis_host: ra::ide::AnalysisHost,
    pub crate_graph: ra::ide::CrateGraphBuilder,
    pub vfs: ra::vfs::Vfs,
}

impl RAFrontend {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn apply_changes(&mut self, crate_graph_changed: bool) {
        let mut change = ra::hir::ChangeWithProcMacros::default();
        let mut directory_structure_changed = false;

        for (file_id, changed_file) in self.vfs.take_changes() {
            use ra::vfs::Change;
            if matches!(changed_file.change, Change::Create(_, _) | Change::Delete) {
                directory_structure_changed = true;
            }
            let contents = match changed_file.change {
                Change::Create(bytes, _) => String::from_utf8(bytes).ok(),
                Change::Modify(bytes, _) => String::from_utf8(bytes).ok(),
                Change::Delete => None,
            };
            change.change_file(file_id, contents);
        }

        if directory_structure_changed {
            change.set_roots(
                ra::vfs::file_set::FileSetConfig::default()
                    .partition(&self.vfs)
                    .into_iter()
                    .map(ra::ide::SourceRoot::new_local)
                    .collect(),
            );
        }
        if crate_graph_changed {
            change.set_crate_graph(self.crate_graph.clone());
        }

        self.analysis_host.apply_change(change);
    }

    pub fn analysis(&self) -> ra::ide::Analysis {
        self.analysis_host.analysis()
    }

    pub fn db(&self) -> &ra::ide::RootDatabase {
        self.analysis_host.raw_database()
    }

    pub fn source(&self, krate: ra::hir::Crate) -> Source {
        Source(self, krate)
    }
}

pub struct Source<'a>(&'a RAFrontend, ra::hir::Crate);

impl of::Source for Source<'_> {
    fn declare<DB: ob::DeclarationBackend>(&self, backend: &mut DB) {
        let db = self.0.db();
        for module in self.1.modules(db) {
            for decl in module.declarations(db) {
                self.declare_symbol(backend, decl)
            }
        }
    }
}

impl Source<'_> {
    fn declare_symbol<DB: ob::DeclarationBackend>(
        &self,
        backend: &mut DB,
        decl: ra::hir::ModuleDef,
    ) {
        let db = self.0.db();
        match decl {
            ra::hir::ModuleDef::Module(module) => todo!(),
            ra::hir::ModuleDef::Function(function) => {
                let params = function
                    .assoc_fn_params(db)
                    .iter()
                    .map(|param| {
                        (
                            param.name(db).map(|name| name.as_str().into()),
                            self.0.convert_type(backend, param.ty()),
                        )
                    })
                    .collect::<Vec<_>>();
                let ret = self.0.convert_type(backend, &function.ret_type(db));
                backend.function(function.name(db).as_str().into(), &params, &ret)
            }
            ra::hir::ModuleDef::Adt(adt) => todo!(),
            ra::hir::ModuleDef::Variant(variant) => todo!(),
            ra::hir::ModuleDef::Const(_) => todo!(),
            ra::hir::ModuleDef::Static(_) => todo!(),
            ra::hir::ModuleDef::Trait(_) => todo!(),
            ra::hir::ModuleDef::TraitAlias(trait_alias) => todo!(),
            ra::hir::ModuleDef::TypeAlias(type_alias) => todo!(),
            ra::hir::ModuleDef::BuiltinType(builtin_type) => todo!(),
            ra::hir::ModuleDef::Macro(_) => todo!(),
        }
    }
}
