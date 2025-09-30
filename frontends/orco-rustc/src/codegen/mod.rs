use crate::TyCtxt;
use orco::DefinitionBackend as Backend;

pub mod function;

pub fn define(tcx: TyCtxt<'_>, backend: &mut impl Backend, items: &rustc_middle::hir::ModuleItems) {
    for item in items.free_items() {
        let item = tcx.hir_item(item);

        use rustc_hir::ItemKind as IK;
        match item.kind {
            IK::ExternCrate(..) => (),
            IK::Use(..) => (),
            IK::Static(..) => (),
            IK::Const(..) => (),
            IK::Fn { .. } => function::define(tcx, backend, item.owner_id.def_id),
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
    }
}
