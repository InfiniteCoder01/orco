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

/// Extraction and conversion of names from HIR to orco::Symbol
pub mod names;

/// Type declaration,
/// useful for generating bindings
pub mod types;

/// rustc backend implementation
pub mod rustc_backend;

/// Code generation is used to define functions and other items
pub mod codegen;

use orco::Backend;
use rustc_middle::ty::TyCtxt;

/// Define a function from MIR by [`rustc_hir::def_id::LocalDefId`].
/// The function MUST have a body.
pub fn function(tcx: TyCtxt, backend: &impl Backend, key: rustc_hir::def_id::LocalDefId) {
    let name = names::convert_path(tcx, key.to_def_id());
    let sig = tcx.fn_sig(key).skip_binder().skip_binder(); // TODO: Generics
    let body = tcx.hir_body_owned_by(key);

    let mut params = Vec::with_capacity(sig.inputs().len());
    for (i, ty) in sig.inputs().iter().enumerate() {
        let name = names::pat_name(body.params[i].pat);
        params.push((name, types::convert(tcx, backend, *ty)));
    }

    let codegen = backend.function(name, params, types::convert(tcx, backend, sig.output()));
    codegen::body(tcx, backend, codegen, tcx.optimized_mir(key));
}

/// Declare a foregin function.
/// Pulls argument names from the slice,
/// since foreign functions don't have a body.
pub fn foreign_function(
    tcx: TyCtxt,
    backend: &impl Backend,
    key: rustc_hir::def_id::DefId,
    idents: &[Option<rustc_span::Ident>],
) {
    let name = names::convert_path(tcx, key);
    let sig = tcx.fn_sig(key).skip_binder().skip_binder(); // TODO: Generics

    let mut params = Vec::with_capacity(sig.inputs().len());
    for (i, ty) in sig.inputs().iter().enumerate() {
        params.push((
            idents[i].map(|ident| ident.as_str().into()),
            types::convert(tcx, backend, *ty),
        ));
    }

    use orco::BodyCodegen;
    backend
        .function(name, params, types::convert(tcx, backend, sig.output()))
        .external();
}

/// Declare a struct type from MIR by [`rustc_hir::def_id::LocalDefId`].
pub fn struct_(tcx: TyCtxt, backend: &impl Backend, key: rustc_hir::def_id::DefId) {
    // TODO: Generics
    let name = names::convert_path(tcx, key);
    let adt = tcx.adt_def(key);
    let orco_ty = orco::Type::Struct(
        // TODO: Default values???
        adt.variants()
            .iter()
            .next()
            .unwrap()
            .fields
            .iter()
            .map(|field| {
                (
                    field.name.as_str().into(),
                    types::convert(tcx, backend, tcx.type_of(field.did).instantiate_identity()),
                )
            })
            .collect::<Vec<_>>(),
    );
    let generics = tcx.generics_of(key);
    if generics.is_empty() {
        backend.type_(name, orco_ty);
    } else {
        backend
            .generic(
                generics
                    .own_params
                    .iter()
                    .map(|param| param.name.as_str().into())
                    .collect(),
            )
            .type_(name, orco_ty);
    }
}

/// Define all the items using the backend provided.
/// See [`TyCtxt::hir_crate_items`]
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
                IK::Fn { .. } => function(tcx, *backend, item.owner_id.def_id),
                IK::Macro(..) => (),
                IK::Mod(..) => (),
                IK::ForeignMod { .. } => (),
                IK::GlobalAsm { .. } => (),
                IK::TyAlias(..) => (),
                IK::Enum(..) => (),
                IK::Struct(..) => struct_(tcx, *backend, item.owner_id.to_def_id()),
                IK::Union(..) => (),
                IK::Trait(..) => (),
                IK::TraitAlias(..) => (),
                IK::Impl(..) => (),
            }
            Ok(())
        })
        .unwrap();

    items.par_impl_items(|_| todo!()).unwrap();

    items
        .par_foreign_items(|item| {
            let item = tcx.hir_foreign_item(item);
            use rustc_hir::ForeignItemKind as FIK;
            match item.kind {
                FIK::Fn(_, idents, _) => {
                    foreign_function(tcx, *backend, item.owner_id.to_def_id(), idents)
                }
                FIK::Static(..) => todo!(),
                FIK::Type => todo!(),
            }
            Ok(())
        })
        .unwrap();
}

/// This is the entrypoint for a hot plugged rustc_codegen_orco
#[unsafe(no_mangle)]
pub fn __rustc_codegen_backend() -> Box<dyn rustc_codegen_ssa::traits::CodegenBackend> {
    Box::new(rustc_backend::OrcoCodegenBackend)
}
