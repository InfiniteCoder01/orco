use crate::TyCtxt;

/// Convert path to [`orco::Symbol`]
pub fn convert_path(tcx: TyCtxt, def_id: rustc_hir::def_id::DefId) -> String {
    let path = tcx.def_path(def_id);
    let mut s = tcx.crate_name(path.krate).to_string();
    s.reserve(path.data.len() * 16 + 16);

    for component in &path.data {
        if matches!(
            component.data,
            rustc_hir::definitions::DefPathData::ForeignMod
        ) {
            s.clear()
        } else {
            if !s.is_empty() {
                s.push_str("::")
            }
            s.push_str(component.as_sym(true).as_str());
        }
    }

    s
}

/// Extract the pattern name, if there is one concrete name
pub fn pat_name(pat: &rustc_hir::Pat) -> Option<String> {
    use rustc_hir::PatKind as PK;
    match pat.kind {
        PK::Missing => None,
        PK::Wild => None,
        PK::Binding(_, _, ident, _) => Some(ident.as_str().to_owned()),
        PK::Struct(..) => None,
        PK::TupleStruct(..) => None,
        PK::Or(..) => None,
        PK::Never => None,
        PK::Tuple(..) => None,
        PK::Box(pat) => pat_name(pat),
        PK::Deref(pat) => pat_name(pat),
        PK::Ref(pat, ..) => pat_name(pat),
        PK::Expr(..) => None,
        PK::Guard(pat, _) => pat_name(pat),
        PK::Range(..) => None,
        PK::Slice(..) => None, // Maybe use the `slice`?
        PK::Err(_) => None,
    }
}
