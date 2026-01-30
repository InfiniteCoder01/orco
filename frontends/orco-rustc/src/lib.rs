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

/// Type conversion
pub mod types;

/// rustc backend implementation
pub mod rustc_backend;

/// Code generation is used to define functions and other items
pub mod codegen;
pub use codegen::codegen;

use orco::DeclarationBackend;
use rustc_middle::ty::TyCtxt;

fn cvt_generics(generics: &rustc_middle::ty::Generics) -> Vec<String> {
    generics
        .own_params
        .iter()
        .map(|param| param.name.as_str().to_owned())
        .collect()
}

macro_rules! declare_w_generics {
    ($tcx:ident $backend:ident $key:ident $decl:block) => {
        let generics = $tcx.generics_of($key);
        if generics.is_empty() $decl
        else {
            let backend = $backend.generic(cvt_generics(generics));
            let $backend = &backend;
            $decl
        }
    };
}

/// Declare a function from MIR by [`rustc_hir::def_id::LocalDefId`].
/// The function MUST have a body. For bodyless functions, see [foreign_function]
pub fn function(
    tcx: TyCtxt,
    backend: &impl DeclarationBackend,
    key: rustc_hir::def_id::LocalDefId,
) {
    let name = names::convert_path(tcx, key.to_def_id()).into();
    let sig = tcx.fn_sig(key).instantiate_identity().skip_binder();
    let body = tcx.hir_body_owned_by(key);

    let mut params = Vec::with_capacity(sig.inputs().len());
    for (i, ty) in sig.inputs().iter().enumerate() {
        let name = names::pat_name(body.params[i].pat);
        params.push((name, types::convert(tcx, *ty)));
    }

    declare_w_generics!(tcx backend key {
        backend.function(name, params, types::convert(tcx, sig.output()));
    });
}

/// Declare a foregin function.
/// Pulls argument names from the slice,
/// since foreign functions don't have a body.
pub fn foreign_function(
    tcx: TyCtxt,
    backend: &impl DeclarationBackend,
    key: rustc_hir::def_id::DefId,
    idents: &[Option<rustc_span::Ident>],
) {
    let name = names::convert_path(tcx, key).into();
    let sig = tcx.fn_sig(key).instantiate_identity().skip_binder();

    let mut params = Vec::with_capacity(sig.inputs().len());
    for (i, ty) in sig.inputs().iter().enumerate() {
        params.push((
            idents[i].map(|ident| ident.as_str().to_owned()),
            types::convert(tcx, *ty),
        ));
    }

    declare_w_generics!(tcx backend key {
        backend.function(name, params, types::convert(tcx, sig.output()));
    });
}

/// Declare a struct type from MIR by [`rustc_hir::def_id::LocalDefId`].
pub fn struct_(tcx: TyCtxt, backend: &impl DeclarationBackend, key: rustc_hir::def_id::DefId) {
    let name = names::convert_path(tcx, key).into();
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
                let name = field.name.to_string();
                (
                    if name.chars().next().is_none_or(|c| c.is_ascii_digit()) {
                        None
                    } else {
                        Some(name)
                    },
                    types::convert(tcx, tcx.type_of(field.did).instantiate_identity()),
                )
            })
            .collect::<Vec<_>>(),
    );

    declare_w_generics!(tcx backend key {
        backend.type_(name, orco_ty);
    });
}

/// Declare all the items using the backend provided.
/// See [`TyCtxt::hir_crate_items`]
pub fn declare(
    tcx: TyCtxt<'_>,
    backend: &impl DeclarationBackend,
    items: &rustc_middle::hir::ModuleItems,
) {
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

    items
        .par_impl_items(|item| {
            let item = tcx.hir_impl_item(item);
            let generics = tcx.generics_of(tcx.parent(item.owner_id.to_def_id()));
            let backend = backend.generic(cvt_generics(generics));

            use rustc_hir::ImplItemKind as IIK;
            // TODO: All of theese
            match item.kind {
                IIK::Const(..) => (),
                IIK::Fn(..) => function(tcx, &backend, item.owner_id.def_id),
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
