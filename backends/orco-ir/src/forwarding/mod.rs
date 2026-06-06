use crate::ir;
use orco::codegen as oc;

mod expression;
mod statements;

impl super::Backend<'_> {
    /// Declare all symbols from this IR in another [`orco::DeclarationBackend`]
    pub fn declare<'a>(&self, backend: &impl orco::DeclarationBackend<'a>) {
        self.types.iter_sync(|name, ty| {
            backend.type_(*name, ty.clone());
            true
        });

        self.functions.iter_sync(|name, signature| {
            backend.function(
                *name,
                signature.params.clone(),
                signature.return_type.clone(),
                signature.attrs.clone(),
            );
            true
        });
    }

    /// Codegen all functions in another [`orco::CodegenBackend`]
    pub fn codegen(&self, backend: &impl orco::CodegenBackend) {
        self.function_definitions.iter_sync(|name, body| {
            let signature = self.functions.get_sync(name).unwrap();
            let args = (0..signature.params.len())
                .map(oc::Variable)
                .collect::<Vec<_>>();
            body.codegen(
                &mut backend.function(*name),
                &args,
                oc::BodyCodegen::return_,
            );
            true
        });
    }

    /// Inline-codegen one function into [`oc::BodyCodegen`]
    pub fn inline_call(
        &self,
        codegen: &mut impl oc::BodyCodegen,
        name: orco::Symbol,
        args: Vec<oc::Value>,
    ) -> Option<oc::Value> {
        // TODO: IMPORTANT! Inline inner function calls and other dependencies on this backend
        let signature = self
            .functions
            .get_sync(&name)
            .unwrap_or_else(|| panic!("trying to inline an undeclared function {name}"));
        let body = self
            .function_definitions
            .get_sync(&name)
            .unwrap_or_else(|| panic!("trying to inline an undefined function {name}"));

        let args = args
            .into_iter()
            .map(|arg| codegen.mk_tmp(arg))
            .collect::<Vec<_>>();
        let retvar = signature
            .return_type
            .clone()
            .map(|rt| codegen.declare_var(rt, Some("_retval")));

        body.codegen(codegen, &args, |cg, value| {
            if let (Some(retval), Some(value)) = (retvar, value) {
                cg.assign(retval.into(), value);
            }
            // TODO: CONTROL FLOW
        });
        retvar.map(|rv| codegen.read(rv.into()))
    }
}

impl ir::Body {
    /// Codegen this body into another [`oc::BodyCodegen`],
    /// mapping all argument variables to `args` (types must be the same).
    pub fn codegen<CG: oc::BodyCodegen>(
        &self,
        codegen: &mut CG,
        args: &[oc::Variable],
        mut codegen_return: impl FnMut(&mut CG, Option<oc::Value>),
    ) {
        let mut variable_map = Vec::with_capacity(self.variables.len());
        for (idx, variable) in self.variables.iter().enumerate() {
            if variable.arg {
                variable_map.push(args[idx]);
            } else {
                variable_map
                    .push(codegen.declare_var(variable.ty.clone(), variable.name.as_deref()))
            }
        }

        use oc::AcfCodegen as _;
        let mut label_map = Vec::with_capacity(self.labels.len());
        let mut statement_idx_to_label = std::collections::HashMap::new();
        for label in &self.labels {
            let backend_label = *label_map.push_mut(codegen.acf().alloc_label());
            statement_idx_to_label.insert(label, backend_label);
        }

        for (idx, statement) in self.statements.iter().enumerate() {
            if let Some(label) = statement_idx_to_label.get(&idx) {
                codegen.acf().label(*label);
            }

            if let ir::Statement::Return(value) = statement {
                let value = value
                    .as_ref()
                    .map(|value| value.codegen(codegen, &|variable| variable_map[variable.0]));
                codegen_return(codegen, value);
                continue;
            }

            statement.codegen(codegen, &|variable| variable_map[variable.0], |label| {
                label_map[label.0]
            });
        }
    }
}
