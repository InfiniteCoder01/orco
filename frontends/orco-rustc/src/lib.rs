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

use rustc_middle::ty::TyCtxt;

/// Extraction and conversion of names from HIR to `orco::Symbol`
pub mod names;

/// Type conversion
pub mod types;

/// rustc backend implementation
pub mod rustc_backend;

/// Symbol declaration routines
pub mod symbols;

/// Code generation is used to define functions and other items
pub mod codegen;
pub use codegen::codegen;

// /// Intrinsic implementations
// pub mod intrinsics;
// pub use intrinsics::intrinsics;

/// Base context for all declaration/codegen operations
#[allow(missing_docs)]
pub struct Context<'tcx, 'b, B> {
    pub tcx: TyCtxt<'tcx>,
    pub backend: &'b B,
}

impl<B> Copy for Context<'_, '_, B> {}
impl<B> Clone for Context<'_, '_, B> {
    fn clone(&self) -> Self {
        Self {
            tcx: self.tcx,
            backend: self.backend,
        }
    }
}

impl<B> Context<'_, '_, B> {
    /// Shorthand for calling [`names::convert_path`]
    pub fn convert_path(
        self,
        key: impl rustc_middle::query::IntoQueryKey<rustc_hir::def_id::DefId>,
    ) -> orco::Symbol {
        names::convert_path(self.tcx, key.into_query_key()).into()
    }

    /// Shorthand for calling [`types::convert`]
    pub fn convert_ty(self, ty: rustc_middle::ty::Ty) -> Option<orco::Type> {
        types::convert(self.tcx, ty)
    }

    /// Shorthand for calling [`types::convert_generic_params`]
    pub fn convert_generics(
        self,
        key: impl rustc_middle::query::IntoQueryKey<rustc_hir::def_id::DefId>,
    ) -> Vec<orco::Type> {
        types::convert_generic_params(self.tcx, key.into_query_key())
    }
}

/// Declare all the items using the backend provided.
/// See [`TyCtxt::hir_crate_items`]
pub fn declare<B>(tcx: TyCtxt, backend: &B, items: &rustc_middle::hir::ModuleItems)
where
    B: orco::DeclarationBackend + Send + Sync,
{
    let backend = rustc_data_structures::sync::IntoDynSyncSend(backend);
    items
        .par_items(|item| {
            let item = tcx.hir_item(item);
            let ctx = Context {
                tcx,
                backend: *backend,
            };

            use rustc_hir::ItemKind as IK;
            // TODO: All of theese
            match item.kind {
                IK::ExternCrate(..) => (),
                IK::Use(..) => (),
                IK::Static(..) => (),
                IK::Const(..) => (),
                IK::Fn { .. } => ctx.function(item.owner_id.def_id),
                IK::Macro(..) => (),
                IK::Mod(..) => (),
                IK::ForeignMod { .. } => (),
                IK::GlobalAsm { .. } => (),
                IK::TyAlias(..) => (),
                IK::Enum(..) => (),
                IK::Struct(..) => ctx.struct_(item.owner_id.to_def_id()),
                IK::Union(..) => (),
                IK::Trait { items, .. } => {
                    for item in items {
                        use rustc_hir::TraitItemKind as TIK;
                        match ctx.tcx.hir_trait_item(*item).kind {
                            TIK::Fn(_, rustc_hir::TraitFn::Required(idents)) => {
                                ctx.function_decl(item.owner_id.to_def_id(), idents)
                            }
                            TIK::Fn(_, rustc_hir::TraitFn::Provided(..)) => {
                                ctx.function(item.owner_id.def_id)
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

            let ctx = Context {
                tcx,
                backend: *backend,
            };

            use rustc_hir::ImplItemKind as IIK;
            // TODO: All of theese
            match item.kind {
                IIK::Const(..) => (),
                IIK::Fn(..) => ctx.function(item.owner_id.def_id),
                IIK::Type(..) => (),
            }

            Ok(())
        })
        .unwrap();

    items
        .par_foreign_items(|item| {
            let item = tcx.hir_foreign_item(item);
            let ctx = Context {
                tcx,
                backend: *backend,
            };

            use rustc_hir::ForeignItemKind as FIK;
            match item.kind {
                FIK::Fn(_, idents, _) => ctx.function_decl(item.owner_id.to_def_id(), idents),
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
