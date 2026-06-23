//! rustc frontend for orco
#![feature(rustc_private)]
#![warn(missing_docs)]

extern crate rustc_ast;
extern crate rustc_codegen_ssa;
extern crate rustc_const_eval;
extern crate rustc_data_structures;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_metadata;
extern crate rustc_middle;
extern crate rustc_public;
extern crate rustc_session;
extern crate rustc_span;
extern crate tracing;

/// Extraction and conversion of names from HIR to `orco::Symbol`
pub mod names;

/// Type conversion
pub mod types;

/// rustc backend implementation
pub mod rustc_backend;

/// Intrinsic implementations
pub mod intrinsics;
pub use intrinsics::intrinsics;

/// Code generation is used to define functions and other items
pub mod codegen;
pub use codegen::codegen;

use orco::DeclarationBackend;
use rustc_middle::ty::TyCtxt;

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

/// Declare a function from MIR by [`rustc_hir::def_id::LocalDefId`].
/// The function MUST have a body. For bodyless functions, see [`foreign_function`]
pub fn function<'a, 'tcx: 'a, B>(tcx: TyCtxt<'tcx>, backend: &B, key: rustc_hir::def_id::LocalDefId)
where
    B: DeclarationBackend<'a> + orco::CodegenBackend,
{
    let attrs = convert_fn_attrs(tcx.codegen_fn_attrs(key));
    let sig = tcx.fn_sig(key).instantiate_identity().skip_binder();
    let body = tcx.hir_body_owned_by(key);

    types::wrap_generics(
        tcx,
        backend,
        key.to_def_id(),
        key.to_def_id(),
        "",
        move |tcx, backend, name, map| {
            let mut params = Vec::with_capacity(sig.inputs().len());
            for (i, ty) in sig.inputs().iter().enumerate() {
                let name = names::pat_name(body.params[i].pat);
                let Some(ty) = types::convert(tcx, backend, *ty, map) else {
                    continue;
                };
                params.push((name, ty));
            }

            backend.function(
                name.into(),
                params,
                types::convert(tcx, backend, sig.output(), map),
                attrs.clone(),
            );
        },
    );
}

/// Declare a foregin function.
/// Pulls argument names from the slice,
/// since foreign functions (or unimplemented trait functions) don't have a body.
pub fn function_decl<'a, 'tcx: 'a>(
    tcx: TyCtxt<'tcx>,
    backend: &impl DeclarationBackend<'a>,
    key: rustc_hir::def_id::DefId,
    idents: &'tcx [Option<rustc_span::Ident>],
) {
    let attrs = convert_fn_attrs(tcx.codegen_fn_attrs(key));
    let sig = tcx.fn_sig(key).instantiate_identity().skip_binder();

    types::wrap_generics(
        tcx,
        backend,
        key,
        key,
        "",
        move |tcx, backend, name, map| {
            let mut params = Vec::with_capacity(sig.inputs().len());
            for (i, ty) in sig.inputs().iter().enumerate() {
                let Some(ty) = types::convert(tcx, backend, *ty, map) else {
                    continue;
                };
                params.push((idents[i].map(|ident| ident.as_str().to_owned()), ty));
            }

            backend.function(
                name.into(),
                params,
                types::convert(tcx, backend, sig.output(), map),
                attrs.clone(),
            );
        },
    );
}

/// Declare a struct type from MIR by [`rustc_hir::def_id::LocalDefId`].
pub fn struct_<'a, 'tcx: 'a>(
    tcx: TyCtxt<'tcx>,
    backend: &impl DeclarationBackend<'a>,
    key: rustc_hir::def_id::DefId,
) {
    let adt = tcx.adt_def(key);
    let variant = adt.variants().iter().next().unwrap();
    types::wrap_generics(
        tcx,
        backend,
        key,
        key,
        "",
        move |tcx, backend, name, map| {
            let mut fields = Vec::with_capacity(variant.fields.len());
            for field in &variant.fields {
                let name = field.name.to_string();
                let Some(ty) = types::convert(
                    tcx,
                    backend,
                    tcx.type_of(field.did)
                        .instantiate_identity()
                        .skip_norm_wip(),
                    map,
                ) else {
                    continue;
                };
                fields.push((
                    match name.chars().next() {
                        Some(c) if !c.is_ascii_digit() => Some(name),
                        _ => None,
                    },
                    ty,
                ));
            }
            backend.type_(name.into(), orco::Type::Struct { fields });
        },
    );
}

/// Declare all the items using the backend provided.
/// See [`TyCtxt::hir_crate_items`]
pub fn declare<'a, 'tcx: 'a, B>(
    tcx: TyCtxt<'tcx>,
    backend: &B,
    items: &rustc_middle::hir::ModuleItems,
) where
    B: DeclarationBackend<'a> + orco::CodegenBackend,
{
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
                IK::Fn { .. } => function(tcx, *backend, item.owner_id.def_id),
                IK::Macro(..) => (),
                IK::Mod(..) => (),
                IK::ForeignMod { .. } => (),
                IK::GlobalAsm { .. } => (),
                IK::TyAlias(..) => (),
                IK::Enum(..) => (),
                IK::Struct(..) => struct_(tcx, *backend, item.owner_id.to_def_id()),
                IK::Union(..) => (),
                IK::Trait { items, .. } => {
                    for item in items {
                        use rustc_hir::TraitItemKind as TIK;
                        match tcx.hir_trait_item(*item).kind {
                            TIK::Fn(_, rustc_hir::TraitFn::Required(idents)) => {
                                function_decl(tcx, *backend, item.owner_id.to_def_id(), idents)
                            }
                            TIK::Fn(_, rustc_hir::TraitFn::Provided(..)) => {
                                function(tcx, *backend, item.owner_id.def_id)
                            }
                            _ => (),
                        }
                    }
                }
                IK::TraitAlias(..) => (),
                IK::Impl(..) => (),
            }
            Ok(())
        })
        .unwrap();

    items
        .par_impl_items(|item| {
            let item = tcx.hir_impl_item(item);
            if matches!(item.impl_kind, rustc_hir::ImplItemImplKind::Trait { .. }) {
                return Ok(());
            }

            use rustc_hir::ImplItemKind as IIK;
            // TODO: All of theese
            match item.kind {
                IIK::Const(..) => (),
                IIK::Fn(..) => function(tcx, *backend, item.owner_id.def_id),
                IIK::Type(..) => (),
            }

            Ok(())
        })
        .unwrap();

    items
        .par_foreign_items(|item| {
            let item = tcx.hir_foreign_item(item);
            use rustc_hir::ForeignItemKind as FIK;
            match item.kind {
                FIK::Fn(_, idents, _) => {
                    function_decl(tcx, *backend, item.owner_id.to_def_id(), idents)
                }
                FIK::Static(..) => todo!(),
                FIK::Type => todo!(),
            }
            Ok(())
        })
        .unwrap();
}

/// This is the entrypoint for a hot plugged `rustc_codegen_orco`
#[unsafe(no_mangle)]
pub fn __rustc_codegen_backend() -> Box<dyn rustc_codegen_ssa::traits::CodegenBackend> {
    Box::new(rustc_backend::OrcoCodegenBackend)
}
