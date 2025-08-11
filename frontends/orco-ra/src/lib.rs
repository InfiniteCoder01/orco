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

fn traverse_defs(
    db: &impl ra::hir::db::DefDatabase,
    def_map: &ra::hir::DefMap,
    callback: &mut impl FnMut(ra::hir::ModuleDefId),
) {
    for (_, module) in def_map.modules() {
        for decl in module.scope.declarations() {
            callback(decl);

            use ra::def::DefWithBodyId;
            use ra::hir::ModuleDefId;
            let decl = match decl {
                ModuleDefId::FunctionId(id) => DefWithBodyId::FunctionId(id),
                ModuleDefId::StaticId(id) => DefWithBodyId::StaticId(id),
                ModuleDefId::ConstId(id) => DefWithBodyId::ConstId(id),
                ModuleDefId::EnumVariantId(id) => DefWithBodyId::VariantId(id),
                _ => continue,
            };
            for (_, def_map) in db.body(decl).blocks(db) {
                traverse_defs(db, def_map, callback);
            }
        }
    }
}

impl of::Source for Source<'_> {
    fn declare(&self, backend: &mut impl ob::DeclarationBackend) {
        let db = self.0.db();
        let def_map = ra::hir::crate_def_map(db, self.1.into());
        traverse_defs(db, def_map, &mut |decl| declare_symbol(backend, db, decl));
    }

    fn define(&self, backend: &mut impl ob::DefinitionBackend) {
        let db = self.0.db();
        let def_map = ra::hir::crate_def_map(db, self.1.into());
        traverse_defs(db, def_map, &mut |decl| define_symbol(backend, db, decl));
    }
}

fn declare_symbol<DB>(
    backend: &mut impl ob::DeclarationBackend,
    db: &DB,
    decl: ra::hir::ModuleDefId,
) where
    DB: ra::hir::db::DefDatabase + ra::hir::db::HirDatabase,
{
    use ra::hir::ModuleDefId;
    match decl {
        ModuleDefId::ModuleId(..) => (),
        ModuleDefId::FunctionId(func) => {
            let sig = db.function_signature(func);
            let sig_ty = db.callable_item_signature(func.into());
            let sig_ty = sig_ty.skip_binders(); // TODO: substitution of generics

            let body = db.body(func.into());
            let cvt_param = |(idx, param)| {
                (
                    if let (0, Some(param)) = (idx, body.self_param) {
                        Some(param)
                    } else {
                        let mut pat = body.params[idx - body.self_param.is_some() as usize];
                        loop {
                            use ra::def::hir::Pat;
                            pat = match body[pat] {
                                Pat::Bind { id, .. } => break Some(id),
                                Pat::Ref { pat, .. } => pat,
                                Pat::Box { inner } => inner,
                                _ => break None,
                            }
                        }
                    }
                    .map(|binding| body[binding].name.as_str().into()),
                    types::convert(backend, param),
                )
            };
            let params = sig_ty
                .params()
                .iter()
                .enumerate()
                .map(cvt_param)
                .collect::<Vec<_>>();
            let ret = types::convert(backend, sig_ty.ret());
            backend.declare_function(sig.name.as_str().into(), &params, &ret)
        }
        ModuleDefId::AdtId(..) => todo!(),
        ModuleDefId::EnumVariantId(..) => todo!(),
        ModuleDefId::ConstId(..) => todo!(),
        ModuleDefId::StaticId(..) => todo!(),
        ModuleDefId::TraitId(..) => todo!(),
        ModuleDefId::TraitAliasId(..) => todo!(),
        ModuleDefId::TypeAliasId(..) => todo!(),
        ModuleDefId::BuiltinType(..) => todo!(),
        ModuleDefId::MacroId(..) => todo!(),
    }
}

fn define_symbol<DB>(backend: &mut impl ob::DefinitionBackend, db: &DB, decl: ra::hir::ModuleDefId)
where
    DB: ra::hir::db::DefDatabase + ra::hir::db::HirDatabase,
{
    match decl {
        ra_ap_hir::ModuleDefId::ModuleId(..) => (),
        ra_ap_hir::ModuleDefId::FunctionId(func) => {
            let id = ra::def::DefWithBodyId::FunctionId(func);
            let sig = db.function_signature(func);
            let body = db.body(id);
            let mut codegen = backend.define_function(sig.name.as_str().into());

            let mut ctx = codegen::CodegenCtx {
                codegen: &mut codegen,
                store: &body.store,
                inference: db.infer(id),
            };

            let value = ctx.build_expr(body.body_expr);

            // Implicit return
            use ob::Codegen;
            match value {
                codegen::Value::Value(value) => ctx.codegen.return_(Some(value)),
                codegen::Value::Unit => ctx.codegen.return_(None),
                codegen::Value::Never => (),
            }
        }
        ra_ap_hir::ModuleDefId::AdtId(..) => todo!(),
        ra_ap_hir::ModuleDefId::EnumVariantId(..) => todo!(),
        ra_ap_hir::ModuleDefId::ConstId(..) => todo!(),
        ra_ap_hir::ModuleDefId::StaticId(..) => todo!(),
        ra_ap_hir::ModuleDefId::TraitId(..) => todo!(),
        ra_ap_hir::ModuleDefId::TraitAliasId(..) => todo!(),
        ra_ap_hir::ModuleDefId::TypeAliasId(..) => todo!(),
        ra_ap_hir::ModuleDefId::BuiltinType(..) => todo!(),
        ra_ap_hir::ModuleDefId::MacroId(..) => todo!(),
    }
}
