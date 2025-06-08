//! rust-analyzer powered rust frontend for orco

pub mod ra {
    pub use ra_ap_base_db::{CrateOrigin, CrateWorkspaceData};
    pub use ra_ap_hir as hir;
    pub use ra_ap_ide as ide;
    pub use ra_ap_vfs as vfs;
}

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

    pub fn status(&self, file_id: ra::vfs::FileId) -> String {
        self.analysis_host.analysis().status(Some(file_id)).unwrap()
    }
}
