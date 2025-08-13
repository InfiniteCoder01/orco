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

pub mod codegen;
pub mod defs;
pub mod types;

/// rust-analyzer frontend, uses multiple [Sources], one per crate
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
            if matches!(changed_file.change, Change::Create(..) | Change::Delete) {
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

    pub fn source(&self, krate: ra::hir::Crate) -> Source<'_> {
        Source(self, krate)
    }
}

pub struct Source<'a>(&'a RAFrontend, ra::hir::Crate);

impl Source<'_> {
    pub fn files(&self) -> Vec<ra::vfs::FileId> {
        let mut files = Vec::new();
        let db = self.0.db();
        for module in self.1.modules(db) {
            if let Some(file) = module.as_source_file_id(db) {
                files.push(file.file_id(db));
            }
        }
        files
    }
}

impl of::Source for Source<'_> {
    fn declare(&self, backend: &mut impl ob::DeclarationBackend) {
        let db = self.0.db();
        let def_map = ra::hir::crate_def_map(db, self.1.into());
        defs::traverse(db, def_map, &mut |decl| defs::declare(backend, db, decl));
    }

    fn define(&self, backend: &mut impl ob::DefinitionBackend) {
        let db = self.0.db();
        let def_map = ra::hir::crate_def_map(db, self.1.into());
        defs::traverse(db, def_map, &mut |decl| defs::define(backend, db, decl));
    }
}
