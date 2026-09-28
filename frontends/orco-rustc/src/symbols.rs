use rustc_public::{CrateDef as _, CrateDefType as _};

fn convert_fn_attrs(
    attrs: &rustc_middle::middle::codegen_fn_attrs::CodegenFnAttrs,
) -> orco::attrs::FunctionAttributes {
    use orco::attrs as oa;
    use rustc_hir::attrs as ra;
    orco::attrs::FunctionAttributes {
        inlining: match attrs.inline {
            ra::InlineAttr::None => oa::Inlining::Auto,
            ra::InlineAttr::Hint => oa::Inlining::Hint,
            ra::InlineAttr::Always => oa::Inlining::Always,
            ra::InlineAttr::Never => oa::Inlining::Never,
            ra::InlineAttr::Force { .. } => oa::Inlining::Always,
        },
    }
}

/// Declare a function from MIR.
pub fn function(func: rustc_public::ty::FnDef, module: &orco::Module) {
    let sig = func.fn_sig().skip_binder();

    let (attrs, params) = crate::internal(func, |tcx, did| {
        let attrs = convert_fn_attrs(tcx.codegen_fn_attrs(did));
        let idents = tcx.fn_arg_idents(did);

        let mut params = Vec::with_capacity(sig.inputs().len());
        for (ident, ty) in idents.iter().zip(sig.inputs()) {
            params.push((
                ident.map(|ident| ident.as_str().to_owned()),
                crate::ty::convert(*ty),
            ));
        }

        (attrs, params)
    });

    module.functions.pin().insert(
        func.name().into(),
        orco::Function {
            generics: crate::ty::convert_generic_params(&func.generics_of()),
            params,
            return_type: crate::ty::convert(sig.output()),
            attrs,
            body: None,
        }
        .into(),
    );
}

/// Declare a struct type from MIR.
pub fn struct_(adt: rustc_public::ty::AdtDef, module: &orco::Module) {
    let variant = adt.variants().into_iter().next().unwrap();
    let fields = variant
        .fields()
        .into_iter()
        .map(|field| {
            let ty = crate::ty::convert(field.ty());
            (
                match field.name.chars().next() {
                    Some(c) if !c.is_ascii_digit() => Some(field.name),
                    _ => None,
                },
                ty,
            )
        })
        .collect();

    module.types.pin().insert(
        adt.name().into(),
        orco::TypeAlias {
            generics: crate::ty::convert_generic_params(&adt.generics_of()),
            type_: orco::Type::Struct { fields },
        }
        .into(),
    );
}
