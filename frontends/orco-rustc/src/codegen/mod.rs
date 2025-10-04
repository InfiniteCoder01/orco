use crate::TyCtxt;
use orco::DefinitionBackend as Backend;

pub mod function;

pub fn define(tcx: TyCtxt<'_>, backend: &impl Backend, items: &rustc_middle::hir::ModuleItems) {
    let backend = rustc_data_structures::sync::IntoDynSyncSend(backend);
    items
        .par_items(|item| {
            let item = tcx.hir_item(item);

            use rustc_hir::ItemKind as IK;
            // TODO: All of theese
            match item.kind {
                IK::ExternCrate(..) => (),
                IK::Use(..) => (),
                IK::Static(..) => (),
                IK::Const(..) => (),
                IK::Fn { .. } => function::define(tcx, *backend, item.owner_id.to_def_id()),
                IK::Macro(..) => (),
                IK::Mod(..) => (),
                IK::ForeignMod { .. } => (),
                IK::GlobalAsm { .. } => (),
                IK::TyAlias(..) => (),
                IK::Enum(..) => (),
                IK::Struct(..) => (),
                IK::Union(..) => (),
                IK::Trait(..) => (),
                IK::TraitAlias(..) => (),
                IK::Impl(..) => (),
            }
            Ok(())
        })
        .unwrap();
    items.par_impl_items(|_| todo!()).unwrap();
}
