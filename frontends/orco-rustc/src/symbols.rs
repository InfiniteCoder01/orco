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

impl<'cg, 'tcx: 'cg, B> crate::Context<'cg, 'cg, 'tcx, B>
where
    B: orco::DeclarationBackend + Send + Sync,
{
    /// Declare a function from MIR by [`rustc_hir::def_id::LocalDefId`].
    /// The function MUST have a body. For bodyless functions, see [`Self::function_decl`]
    pub fn function(self, key: rustc_hir::def_id::LocalDefId) {
        let attrs = convert_fn_attrs(self.tcx.codegen_fn_attrs(key));
        let sig = self.tcx.fn_sig(key).instantiate_identity().skip_binder();
        let body = self.tcx.hir_body_owned_by(key);

        self.wrap_generic(key.to_def_id(), move |ctx| {
            let mut params = Vec::with_capacity(sig.inputs().len());
            for (i, ty) in sig.inputs().iter().enumerate() {
                let name = crate::names::pat_name(body.params[i].pat);
                let Some(ty) = ctx.convert_ty(*ty) else {
                    continue;
                };
                params.push((name, ty));
            }

            ctx.backend.function(
                ctx.name,
                params,
                ctx.convert_ty(sig.output()),
                attrs.clone(),
            );
        });
    }

    /// Declare a foregin function.
    /// Pulls argument names from the slice,
    /// since foreign functions (or unimplemented trait functions) don't have a body.
    pub fn function_decl(
        self,
        key: rustc_hir::def_id::DefId,
        idents: &'tcx [Option<rustc_span::Ident>],
    ) {
        let attrs = convert_fn_attrs(self.tcx.codegen_fn_attrs(key));
        let sig = self.tcx.fn_sig(key).instantiate_identity().skip_binder();

        self.wrap_generic(key, move |ctx| {
            let mut params = Vec::with_capacity(sig.inputs().len());
            for (i, ty) in sig.inputs().iter().enumerate() {
                let Some(ty) = ctx.convert_ty(*ty) else {
                    continue;
                };
                params.push((idents[i].map(|ident| ident.as_str().to_owned()), ty));
            }

            ctx.backend.function(
                ctx.name,
                params,
                ctx.convert_ty(sig.output()),
                attrs.clone(),
            );
        });
    }

    /// Declare a struct type from MIR by [`rustc_hir::def_id::DefId`].
    pub fn struct_(self, key: rustc_hir::def_id::DefId) {
        let adt = self.tcx.adt_def(key);
        let variant = adt.variants().iter().next().unwrap();
        self.wrap_generic(key, move |ctx| {
            let mut fields = Vec::with_capacity(variant.fields.len());
            for field in &variant.fields {
                let name = field.name.to_string();
                let Some(ty) = ctx.convert_ty(
                    ctx.tcx
                        .type_of(field.did)
                        .instantiate_identity()
                        .skip_norm_wip(),
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
            ctx.backend
                .type_(ctx.name.into(), orco::Type::Struct { fields });
        });
    }

    /// Declare all the items using the backend provided.
    /// See [`TyCtxt::hir_crate_items`]
    pub fn declare(self, items: &rustc_middle::hir::ModuleItems) {
        items
            .par_items(|item| {
                let item = self.tcx.hir_item(item);

                use rustc_hir::ItemKind as IK;
                // TODO: All of theese
                match item.kind {
                    IK::ExternCrate(..) => (),
                    IK::Use(..) => (),
                    IK::Static(..) => (),
                    IK::Const(..) => (),
                    IK::Fn { .. } => self.function(item.owner_id.def_id),
                    IK::Macro(..) => (),
                    IK::Mod(..) => (),
                    IK::ForeignMod { .. } => (),
                    IK::GlobalAsm { .. } => (),
                    IK::TyAlias(..) => (),
                    IK::Enum(..) => (),
                    IK::Struct(..) => self.struct_(item.owner_id.to_def_id()),
                    IK::Union(..) => (),
                    IK::Trait { items, .. } => {
                        for item in items {
                            use rustc_hir::TraitItemKind as TIK;
                            match self.tcx.hir_trait_item(*item).kind {
                                TIK::Fn(_, rustc_hir::TraitFn::Required(idents)) => {
                                    self.function_decl(item.owner_id.to_def_id(), idents)
                                }
                                TIK::Fn(_, rustc_hir::TraitFn::Provided(..)) => {
                                    self.function(item.owner_id.def_id)
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
                let item = self.tcx.hir_impl_item(item);
                if matches!(item.impl_kind, rustc_hir::ImplItemImplKind::Trait { .. }) {
                    return Ok(());
                }

                use rustc_hir::ImplItemKind as IIK;
                // TODO: All of theese
                match item.kind {
                    IIK::Const(..) => (),
                    IIK::Fn(..) => self.function(item.owner_id.def_id),
                    IIK::Type(..) => (),
                }

                Ok(())
            })
            .unwrap();

        items
            .par_foreign_items(|item| {
                let item = self.tcx.hir_foreign_item(item);
                use rustc_hir::ForeignItemKind as FIK;
                match item.kind {
                    FIK::Fn(_, idents, _) => self.function_decl(item.owner_id.to_def_id(), idents),
                    FIK::Static(..) => todo!(),
                    FIK::Type => todo!(),
                }
                Ok(())
            })
            .unwrap();
    }
}
