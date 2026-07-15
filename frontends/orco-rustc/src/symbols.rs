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

impl<B> crate::Context<'_, '_, B>
where
    B: orco::DeclarationBackend,
{
    /// Declare a function from MIR by [`rustc_hir::def_id::LocalDefId`].
    /// The function MUST have a body. For bodyless functions, see [`Self::function_decl`]
    pub fn function(self, key: rustc_hir::def_id::LocalDefId) {
        let attrs = convert_fn_attrs(self.tcx.codegen_fn_attrs(key));
        let sig = self.tcx.fn_sig(key).instantiate_identity().skip_binder();
        let body = self.tcx.hir_body_owned_by(key);

        let mut params = Vec::with_capacity(sig.inputs().len());
        for (i, ty) in sig.inputs().iter().enumerate() {
            let name = crate::names::pat_name(body.params[i].pat);
            let Some(ty) = self.convert_ty(*ty) else {
                continue;
            };
            params.push((name, ty));
        }

        self.backend.function(
            self.convert_path(key),
            self.convert_generics(key),
            params,
            self.convert_ty(sig.output()),
            attrs.clone(),
        );
    }

    /// Declare a foregin function.
    /// Pulls argument names from the slice,
    /// since foreign functions (or unimplemented trait functions) don't have a body.
    pub fn function_decl(
        self,
        key: rustc_hir::def_id::DefId,
        idents: &[Option<rustc_span::Ident>],
    ) {
        let attrs = convert_fn_attrs(self.tcx.codegen_fn_attrs(key));
        let sig = self.tcx.fn_sig(key).instantiate_identity().skip_binder();

        let mut params = Vec::with_capacity(sig.inputs().len());
        for (i, ty) in sig.inputs().iter().enumerate() {
            let Some(ty) = self.convert_ty(*ty) else {
                continue;
            };
            params.push((idents[i].map(|ident| ident.as_str().to_owned()), ty));
        }

        self.backend.function(
            self.convert_path(key),
            self.convert_generics(key),
            params,
            self.convert_ty(sig.output()),
            attrs.clone(),
        );
    }

    /// Declare a struct type from MIR by [`rustc_hir::def_id::DefId`].
    pub fn struct_(self, key: rustc_hir::def_id::DefId) {
        let adt = self.tcx.adt_def(key);
        let variant = adt.variants().iter().next().unwrap();

        let mut fields = Vec::with_capacity(variant.fields.len());
        for field in &variant.fields {
            let name = field.name.to_string();
            let Some(ty) = self.convert_ty(
                self.tcx
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
        self.backend.type_(
            self.convert_path(key),
            self.convert_generics(key),
            orco::Type::Struct { fields },
        );
    }
}
