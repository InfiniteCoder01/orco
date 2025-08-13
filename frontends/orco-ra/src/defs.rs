use crate::{ob, ra};
use ra::hir::db::{DefDatabase, HirDatabase};

pub(crate) fn traverse(
    db: &impl DefDatabase,
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
                traverse(db, def_map, callback);
            }
        }
    }
}

pub(crate) fn declare(
    backend: &mut impl ob::DeclarationBackend,
    db: &impl HirDatabase,
    decl: ra::hir::ModuleDefId,
) {
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
                        Some(body[param].name.as_str().into())
                    } else {
                        let mut pat = body.params[idx - body.self_param.is_some() as usize];
                        loop {
                            use ra::def::hir::Pat;
                            pat = match body[pat] {
                                Pat::Bind { id, .. } => break Some(body[id].name.as_str().into()),
                                Pat::Ref { pat, .. } => pat,
                                Pat::Box { inner } => inner,
                                Pat::Wild => break None,
                                _ => break Some(format!("__param{idx}").into()),
                            }
                        }
                    },
                    crate::types::convert(backend, param),
                )
            };
            let params = sig_ty
                .params()
                .iter()
                .enumerate()
                .map(cvt_param)
                .collect::<Vec<_>>();
            let ret = crate::types::convert(backend, sig_ty.ret());
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

pub(crate) fn define(
    backend: &mut impl ob::DefinitionBackend,
    db: &impl HirDatabase,
    decl: ra::hir::ModuleDefId,
) {
    match decl {
        ra_ap_hir::ModuleDefId::ModuleId(..) => (),
        ra_ap_hir::ModuleDefId::FunctionId(func) => {
            let def = ra::def::DefWithBodyId::FunctionId(func);
            let sig = db.function_signature(func);
            let mut codegen = backend.define_function(sig.name.as_str().into());

            let mut ctx = crate::codegen::CodegenCtx::new(&mut codegen, db, def);
            let value = ctx.build_expr(ctx.body().body_expr);

            // Implicit return
            use crate::codegen::Value;
            use ob::FunctionCodegen as _;
            match value {
                Value::Value(value) => codegen.return_(Some(value)),
                Value::Unit => codegen.return_(None),
                Value::Never => (),
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
