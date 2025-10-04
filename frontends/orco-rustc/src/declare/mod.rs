use crate::TyCtxt;
use orco::DeclarationBackend as Backend;

/// Extraction and conversion of names from HIR to orco::Symbol
pub mod names;
pub use names::{convert_path, pat_name};

/// Convert a type from rust MIR to orco.
/// Pass your backend as the type source
pub fn convert_type(pts: &impl orco::PrimitiveTypeSource, ty: rustc_middle::ty::Ty) -> orco::Type {
    use rustc_middle::ty::{FloatTy, IntTy, TyKind, UintTy};
    match ty.kind() {
        TyKind::Bool => pts.bool(),
        TyKind::Char => todo!(),
        TyKind::Int(IntTy::I8) => pts.int(8, true),
        TyKind::Int(IntTy::I16) => pts.int(16, true),
        TyKind::Int(IntTy::I32) => pts.int(32, true),
        TyKind::Int(IntTy::I64) => pts.int(64, true),
        TyKind::Int(IntTy::I128) => pts.int(128, true),
        TyKind::Int(IntTy::Isize) => pts.size_type(true),
        TyKind::Uint(UintTy::U8) => pts.int(8, false),
        TyKind::Uint(UintTy::U16) => pts.int(16, false),
        TyKind::Uint(UintTy::U32) => pts.int(32, false),
        TyKind::Uint(UintTy::U64) => pts.int(64, false),
        TyKind::Uint(UintTy::U128) => pts.int(128, false),
        TyKind::Uint(UintTy::Usize) => pts.size_type(false),
        TyKind::Float(FloatTy::F16) => pts.float(16),
        TyKind::Float(FloatTy::F32) => pts.float(32),
        TyKind::Float(FloatTy::F64) => pts.float(64),
        TyKind::Float(FloatTy::F128) => pts.float(128),
        TyKind::Adt(..) => todo!(),
        TyKind::Foreign(..) => todo!(),
        TyKind::Str => todo!(),
        TyKind::Array(..) => todo!(),
        TyKind::Pat(..) => todo!(),
        TyKind::Slice(..) => todo!(),
        TyKind::RawPtr(..) => todo!(),
        TyKind::Ref(..) => todo!(),
        TyKind::FnDef(..) => todo!(),
        TyKind::FnPtr(..) => todo!(),
        TyKind::UnsafeBinder(..) => todo!(),
        TyKind::Dynamic(..) => todo!(),
        TyKind::Closure(..) => todo!(),
        TyKind::CoroutineClosure(..) => todo!(),
        TyKind::Coroutine(..) => todo!(),
        TyKind::CoroutineWitness(..) => todo!(),
        TyKind::Never => todo!(),
        TyKind::Tuple(v) if v.is_empty() => pts.unit(),
        TyKind::Tuple(..) => todo!(),
        TyKind::Alias(..) => todo!(),
        TyKind::Param(..) => todo!(),
        TyKind::Bound(..) => todo!(),
        TyKind::Placeholder(..) => todo!(),
        TyKind::Infer(var) => panic!("inference variable {var} found in type"),
        TyKind::Error(..) => orco::Type::Error,
    }
}

/// Declare all the items using the backend provided.
/// See [`TyCtxt::hir_crate_items`]
pub fn declare(tcx: TyCtxt, backend: &mut impl Backend, items: &rustc_middle::hir::ModuleItems) {
    for item in items.free_items() {
        let item = tcx.hir_item(item);

        use rustc_hir::ItemKind as IK;
        match item.kind {
            IK::ExternCrate(..) => (),
            IK::Use(..) => (),
            IK::Static(..) => (),
            IK::Const(..) => (),
            IK::Fn { .. } => declare_function(tcx, backend, item.owner_id.def_id),
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
    for _item in items.impl_items() {
        todo!()
    }
    for item in items.foreign_items() {
        let item = tcx.hir_foreign_item(item);
        use rustc_hir::ForeignItemKind as FIK;
        match item.kind {
            FIK::Fn(_, idents, _) => {
                declare_foreign_function(tcx, backend, item.owner_id.to_def_id(), idents)
            }
            FIK::Static(..) => todo!(),
            FIK::Type => todo!(),
        }
    }
}

/// Declare a function by def_id. The function MUST have a body.
pub fn declare_function(
    tcx: TyCtxt,
    backend: &mut impl Backend,
    key: rustc_hir::def_id::LocalDefId, // TODO: non-local?
) {
    let name = convert_path(tcx, key.to_def_id());
    let sig = tcx.fn_sig(key).skip_binder().skip_binder();
    let body = tcx.hir_body_owned_by(key);

    let mut params = Vec::with_capacity(sig.inputs().len());
    for (i, ty) in sig.inputs().iter().enumerate() {
        let name = pat_name(body.params[i].pat);
        params.push((name, convert_type(backend, *ty)));
    }

    backend.declare_function(name, &params, &convert_type(backend, sig.output()));
}

/// Declare a foregin function
/// Pulls argument names from the slice,
/// since foreign functions don't have a body.
pub fn declare_foreign_function(
    tcx: TyCtxt,
    backend: &mut impl Backend,
    key: rustc_hir::def_id::DefId,
    idents: &[Option<rustc_span::Ident>],
) {
    let name = convert_path(tcx, key);
    let sig = tcx.fn_sig(key).skip_binder().skip_binder();

    let mut params = Vec::with_capacity(sig.inputs().len());
    for (i, ty) in sig.inputs().iter().enumerate() {
        params.push((
            idents[i].map(|ident| ident.as_str().into()),
            convert_type(backend, *ty),
        ));
    }

    backend.declare_function(name, &params, &convert_type(backend, sig.output()));
}
