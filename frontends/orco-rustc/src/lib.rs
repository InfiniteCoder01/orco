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

// /// Code generation is used to define functions and other items
// pub mod codegen;
// pub use codegen::codegen;

// /// Intrinsic implementations
// pub mod intrinsics;
// pub use intrinsics::intrinsics;

/// Base context for all declaration/codegen operations
#[allow(missing_docs)]
pub struct Context<'cg, 'ms, 'tcx, B> {
    pub tcx: TyCtxt<'tcx>,
    pub backend: &'cg B,
    pub server: &'cg orco::MacroServer<'ms>,
}

impl<B> Clone for Context<'_, '_, '_, B> {
    fn clone(&self) -> Self {
        Self {
            tcx: self.tcx,
            backend: self.backend,
            server: self.server,
        }
    }
}

impl<B> Copy for Context<'_, '_, '_, B> {}

/// Context that passed monomorphization.
/// Can be used effectively for declarations
#[allow(missing_docs)]
pub struct MonoContext<'cg, 'ms, 'tcx, B> {
    pub context: Context<'cg, 'ms, 'tcx, B>,
    pub name: orco::Symbol,
    pub map: types::GenericMap<'cg>,
}

impl<B> Clone for MonoContext<'_, '_, '_, B> {
    fn clone(&self) -> Self {
        Self {
            context: self.context,
            name: self.name,
            map: self.map,
        }
    }
}

impl<B> Copy for MonoContext<'_, '_, '_, B> {}

impl<'cg, 'ms, 'tcx, B> std::ops::Deref for MonoContext<'cg, 'ms, 'tcx, B> {
    type Target = Context<'cg, 'ms, 'tcx, B>;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

impl<B> std::ops::DerefMut for MonoContext<'_, '_, '_, B> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.context
    }
}

impl<'cg, 'tcx: 'cg, B: Send + Sync> Context<'cg, 'cg, 'tcx, B> {
    fn wrap_generic(
        self,
        key: rustc_hir::def_id::DefId,
        callback: impl Fn(MonoContext<'_, 'cg, 'tcx, B>) + Send + Sync + 'cg,
    ) {
        let generics = self.tcx.generics_of(key);
        let name = crate::names::convert_path(self.tcx, key);
        if !generics.requires_monomorphization(self.tcx) {
            callback(MonoContext {
                context: self,
                name: name.into(),
                map: types::GenericMap::new(generics, &[]),
            });
        } else {
            let tcx = rustc_data_structures::sync::check_dyn_thread_safe()
            .expect(
                "You have to enable `-Z threads` (f.e. `-Z threads=sync`) to be able to use macros/generics",
            )
            .derive(self.tcx);

            self.server.macro_(
                name.as_str().into(),
                move |args| {
                    let mut name = name.clone();
                    for arg in args {
                        name.push('_');
                        name.push_str(&arg.hashable_name());
                    }

                    callback(MonoContext {
                        context: Context {
                            tcx: *tcx,
                            backend: self.backend,
                            server: self.server,
                        },
                        name: name.into(),
                        map: types::GenericMap::new(generics, args),
                    });
                },
                true,
            );
        }
    }
}

impl<B: orco::DeclarationBackend> MonoContext<'_, '_, '_, B> {
    /// See [types::convert]
    #[inline]
    #[must_use]
    pub fn convert_ty(self, ty: rustc_middle::ty::Ty) -> Option<orco::Type> {
        types::convert(self.context, ty, self.map)
    }
}

/// This is the entrypoint for a hot plugged `rustc_codegen_orco`
#[unsafe(no_mangle)]
pub fn __rustc_codegen_backend() -> Box<dyn rustc_codegen_ssa::traits::CodegenBackend> {
    Box::new(rustc_backend::OrcoCodegenBackend)
}
