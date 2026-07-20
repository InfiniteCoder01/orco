use crate::ir;
use orco::codegen as oc;

mod expression;
mod generics;
mod statements;

impl super::Store {
    /// Declare all symbols from this IR in another [`orco::DeclarationBackend`]
    pub fn declare(&self, backend: &impl orco::DeclarationBackend) {
        for (name, specs) in self.types.pin().iter() {
            for (generics, ty) in specs.pin().iter() {
                backend.type_(*name, generics.clone(), ty.clone());
            }
        }

        for (name, decl) in self.functions.pin().iter() {
            backend.function(
                *name,
                decl.generic_params.clone(),
                decl.signature.params.clone(),
                decl.signature.return_type.clone(),
                decl.signature.attrs.clone(),
            );
        }
    }

    /// Codegen all functions in another [`orco::CodegenBackend`]
    pub fn codegen(&self, backend: &impl orco::CodegenBackend) {
        let decls = self.functions.pin();
        for (name, specs) in self.function_bodies.pin().iter() {
            let decl = decls
                .get(name)
                .unwrap_or_else(|| panic!("BUG: unable to find declaration while defining {name}"));
            let args = (0..decl.signature.params.len())
                .map(oc::Variable)
                .collect::<Vec<_>>();
            for (generics, body) in specs.pin().iter() {
                body.codegen(
                    &mut backend.cg_function(*name, generics.clone()),
                    &args,
                    crate::generics::TypeMap::new(),
                    oc::BodyCodegen::return_,
                );
            }
        }
    }

    /// Inline-codegen one function into [`oc::BodyCodegen`]
    pub fn inline_call(
        &self,
        codegen: &mut impl oc::BodyCodegen,
        name: orco::Symbol,
        generics: &[orco::Type],
        args: Vec<oc::Value>,
    ) -> Option<oc::Value> {
        // TODO: IMPORTANT! Inline inner function calls and other dependencies on this backend
        let decls = self.functions.pin();
        let decl = decls
            .get(&name)
            .unwrap_or_else(|| panic!("trying to inline an undeclared function {name}"));
        let signature = decl.instantiate(self, generics);

        let args = args
            .into_iter()
            .map(|arg| codegen.mk_tmp(arg))
            .collect::<Vec<_>>();
        let retvar = signature.return_type.clone().map(|mut rt| {
            rt.instantiate(
                &crate::generics::match_type_params(&decl.generic_params, generics, self)
                    .unwrap_or_else(|| {
                        panic!(
                            "generics do not match for {name}{}",
                            orco::types::fmt_generics(generics)
                        )
                    }),
            );
            codegen.declare_var(rt, Some("_retval"))
        });

        use orco::codegen::AcfCodegen;
        let return_label = codegen.acf().alloc_label();

        self.get_function_body(name, generics, |body, map| {
            body.codegen(codegen, &args, map, |cg, value| {
                if let (Some(retval), Some(value)) = (retvar, value) {
                    cg.assign(retval.into(), value);
                }
                cg.acf().jump(return_label);
            });
        });

        codegen.acf().label(return_label);
        retvar.map(|rv| codegen.read(rv.into()))
    }
}

/// Context for converting IR to [`oc::BodyCodegen`] calls
struct FwdCtx<'a, CG: oc::BodyCodegen> {
    /// The codegen reference
    cg: &'a mut CG,
    /// Map from type parameters to types
    type_map: crate::generics::TypeMap,
    /// Map from IR variable indices to codegen variables
    variable_map: Vec<oc::Variable>,
    /// Map from IR label indices to codegen labels
    label_map: Vec<oc::Label>,
}

impl ir::Body {
    /// Codegen this body into another [`oc::BodyCodegen`],
    /// mapping all argument variables to `args` (types must be the same).
    pub fn codegen<CG: oc::BodyCodegen>(
        &self,
        codegen: &mut CG,
        args: &[oc::Variable],
        type_map: crate::generics::TypeMap,
        mut codegen_return: impl FnMut(&mut CG, Option<oc::Value>),
    ) {
        let mut ctx = FwdCtx {
            cg: codegen,
            variable_map: Vec::with_capacity(self.variables.len()),
            label_map: Vec::with_capacity(self.labels.len()),
            type_map,
        };

        for (idx, variable) in self.variables.iter().enumerate() {
            if variable.arg {
                ctx.variable_map.push(args[idx]);
            } else {
                ctx.variable_map.push(ctx.cg.declare_var(
                    variable.ty.copy_instantiate(&ctx.type_map),
                    variable.name.as_deref(),
                ))
            }
        }

        use oc::AcfCodegen as _;
        let mut statement_idx_to_label = std::collections::HashMap::new();
        for label in &self.labels {
            let backend_label = *ctx.label_map.push_mut(ctx.cg.acf().alloc_label());
            statement_idx_to_label.insert(label, backend_label);
        }

        for (idx, statement) in self.statements.iter().enumerate() {
            if let Some(label) = statement_idx_to_label.get(&idx) {
                ctx.cg.acf().label(*label);
            }

            if let ir::Statement::Return(expr) = statement {
                let expr = expr.as_ref().map(|expr| ctx.expr(expr));
                codegen_return(ctx.cg, expr);
                continue;
            }

            ctx.stmt(statement);
        }
    }
}
