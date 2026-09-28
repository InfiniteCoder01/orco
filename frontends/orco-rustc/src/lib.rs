//! rustc frontend for orco
#![feature(rustc_private)]
#![feature(f16)]
#![feature(f128)]
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

/// Type conversion
pub mod ty;

/// rustc backend implementation
pub mod rustc_backend;

/// Symbol declaration routines
pub mod symbols;

/// Code generation is used to define functions and other items
pub mod codegen;
pub use codegen::codegen;

// // /// Intrinsic implementations
// // pub mod intrinsics;
// // pub use intrinsics::intrinsics;

/// Declare all the items using the backend provided.
/// See [`TyCtxt::hir_crate_items`]
pub fn declare(crate_: rustc_public::Crate, module: &orco::Module) {
    // TODO: Type aliases
    for adt in crate_.adts() {
        use rustc_public::ty::AdtKind;
        match adt.kind() {
            AdtKind::Enum => todo!(),
            AdtKind::Union => todo!(),
            AdtKind::Struct => symbols::struct_(adt, module),
        }
    }

    for func in crate_.fn_defs() {
        symbols::function(func, module);
    }
}

/// This is the entrypoint for a hot plugged `rustc_codegen_orco`
#[unsafe(no_mangle)]
pub fn __rustc_codegen_backend() -> Box<dyn rustc_codegen_ssa::traits::CodegenBackend> {
    Box::new(rustc_backend::OrcoCodegenBackend)
}

/// An escape hatch to get [`rustc_middle::ty::TyCtxt`] and [`rustc_hir::def_id::DefId`].
pub fn internal<R>(
    item: impl rustc_public::CrateDef,
    callback: impl FnOnce(rustc_middle::ty::TyCtxt, rustc_hir::def_id::DefId) -> R,
) -> R {
    rustc_middle::ty::tls::with_context(|ctxt| {
        let did = rustc_public::rustc_internal::internal(ctxt.tcx, item.def_id());
        callback(ctxt.tcx, did)
    })
}
