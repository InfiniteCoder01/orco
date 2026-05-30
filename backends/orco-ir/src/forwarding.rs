use crate::ir;
use oc::ACFCodegen as _;
use orco::codegen as oc;

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
                .map(|idx| oc::Variable(idx))
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

impl ir::Place {
    /// Convert this place into [`oc::Place`],
    /// while generating code for inner expressions using
    /// [`ir::Expression::codegen`]
    fn codegen(
        &self,
        codegen: &mut impl oc::BodyCodegen,
        map_variable: &impl Fn(oc::Variable) -> oc::Variable,
    ) -> oc::Place {
        match self {
            ir::Place::Variable(variable) => map_variable(*variable).into(),
            ir::Place::Global(symbol) => oc::Place::Global(*symbol),
            ir::Place::Deref(value) => oc::Place::Deref(value.codegen(codegen, map_variable)),
            ir::Place::Field(place, idx) => place.codegen(codegen, map_variable).field(*idx),
        }
    }
}

impl ir::Expression {
    /// Codegen this expression into another [`oc::BodyCodegen`],
    /// mapping all variables
    fn codegen(
        &self,
        codegen: &mut impl oc::BodyCodegen,
        map_variable: &impl Fn(oc::Variable) -> oc::Variable,
    ) -> oc::Value {
        match self {
            Self::IConst(value, size) => codegen.iconst(*value, *size),
            Self::UConst(value, size) => codegen.uconst(*value, *size),
            Self::FConst(value, size) => codegen.fconst(*value, *size),
            Self::BConst(value) => codegen.bconst(*value),
            Self::Read(place) => {
                let place = place.codegen(codegen, map_variable);
                codegen.read(place)
            }
            Self::Reference(place, mutable) => {
                let place = place.codegen(codegen, map_variable);
                codegen.reference(place, *mutable)
            }
            Self::Call(func, args) => {
                let func = func.codegen(codegen, map_variable);
                let args = args
                    .iter()
                    .map(|arg| arg.codegen(codegen, map_variable))
                    .collect();
                codegen
                    .call(func, args)
                    .unwrap_or_else(|| panic!("trying to use value from calling a void function"))
            }

            Self::Intrinsic(intrinsic) => {
                use crate::ir::Intrinsic as I;
                use oc::Intrinsics as IT;
                match intrinsic {
                    I::Add(a, b) => {
                        let a = a.codegen(codegen, map_variable);
                        let b = b.codegen(codegen, map_variable);
                        codegen.intrinsics().add(a, b)
                    }
                    I::Mul(a, b) => {
                        let a = a.codegen(codegen, map_variable);
                        let b = b.codegen(codegen, map_variable);
                        codegen.intrinsics().add(a, b)
                    }
                }
            }
        }
    }
}

impl ir::Statement {
    /// Codegen this statement into another [`oc::BodyCodegen`],
    /// mapping all variables and labels (ACF)
    fn codegen(
        &self,
        codegen: &mut impl oc::BodyCodegen,
        map_variable: &impl Fn(oc::Variable) -> oc::Variable,
        map_label: impl Fn(oc::Label) -> oc::Label,
    ) {
        match self {
            Self::Comment(comment) => codegen.comment(&comment),
            Self::Assign(place, value) => {
                let place = place.codegen(codegen, map_variable);
                let value = value.codegen(codegen, map_variable);
                codegen.assign(place, value)
            }
            Self::Call(func, args) => {
                let func = func.codegen(codegen, map_variable);
                let args = args
                    .iter()
                    .map(|arg| arg.codegen(codegen, map_variable))
                    .collect();
                if let Some(value) = codegen.call(func, args) {
                    codegen.mk_tmp(value);
                }
            }
            Self::Return(value) => {
                let value = value
                    .as_ref()
                    .map(|value| value.codegen(codegen, map_variable));
                codegen.return_(value)
            }

            Self::ACFJump(label) => codegen.acf().jump(map_label(*label)),
            Self::ACFCJump(value, label) => {
                let value = value.codegen(codegen, map_variable);
                codegen.acf().cjump(value, map_label(*label))
            }
        }
    }
}
