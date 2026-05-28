use crate::ir;
use oc::ACFCodegen as _;
use oc::BodyCodegen as _;
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
            body.codegen(&mut backend.function(*name), &args, |cg, value| {
                cg.return_(value)
            });
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
            .map(|rt| codegen.declare_var(rt));

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
                variable_map.push(codegen.declare_var(variable.ty.clone()))
            }
        }

        let mut label_map = Vec::with_capacity(self.labels.len());
        let mut statement_idx_to_label = std::collections::HashMap::new();
        for label in &self.labels {
            let backend_label = *label_map.push_mut(codegen.acf().alloc_label());
            statement_idx_to_label.insert(label, backend_label);
        }

        let mut value_map = Vec::<Option<oc::Value>>::with_capacity(self.statements.len());
        for (idx, statement) in self.statements.iter().enumerate() {
            if let Some(label) = statement_idx_to_label.get(&idx) {
                codegen.acf().label(*label);
            }

            let map_value = |value: &oc::Value| {
                value_map
                    .get_mut(value.0)
                    .and_then(Option::take)
                    .take()
                    .unwrap_or_else(|| panic!("invalid value id {}", value.0))
            };
            if let ir::Statement::Return(value) = statement {
                codegen_return(codegen, value.as_ref().map(map_value));
                value_map.push(None);
                continue;
            }

            let value = statement.codegen(
                codegen,
                |variable| variable_map[variable.0],
                map_value,
                |label| label_map[label.0],
            );
            value_map.push(value);
        }
    }
}

impl ir::Statement {
    /// Codegen this statement into another [`oc::BodyCodegen`],
    /// mapping all variables, values and labels (ACF).
    fn codegen(
        &self,
        codegen: &mut impl oc::BodyCodegen,
        map_variable: impl Fn(oc::Variable) -> oc::Variable,
        mut map_value: impl FnMut(&oc::Value) -> oc::Value,
        map_label: impl Fn(oc::Label) -> oc::Label,
    ) -> Option<oc::Value> {
        fn map_place(
            place: &oc::Place,
            map_variable: impl Fn(oc::Variable) -> oc::Variable,
            mut map_value: impl FnMut(&oc::Value) -> oc::Value,
        ) -> oc::Place {
            match place {
                oc::Place::Variable(variable) => map_variable(*variable).into(),
                oc::Place::Global(symbol) => oc::Place::Global(*symbol),
                oc::Place::Deref(value) => oc::Place::Deref(map_value(value)),
                oc::Place::Field(place, idx) => {
                    map_place(place.as_ref(), map_variable, map_value).field(*idx)
                }
            }
        }

        use crate::ir::Intrinsic as I;
        match self {
            Self::Comment(comment) => {
                codegen.comment(&comment);
                None
            }
            Self::Assign(place, value) => {
                codegen.assign(
                    map_place(place, &map_variable, &mut map_value),
                    map_value(value),
                );
                None
            }
            Self::IConst(value, size) => Some(codegen.iconst(*value, *size)),
            Self::UConst(value, size) => Some(codegen.uconst(*value, *size)),
            Self::FConst(value, size) => Some(codegen.fconst(*value, *size)),
            Self::BConst(value) => Some(codegen.bconst(*value)),
            Self::Read(place) => {
                Some(codegen.read(map_place(place, &map_variable, &mut map_value)))
            }
            Self::Reference(place, mutable) => {
                Some(codegen.reference(map_place(place, &map_variable, &mut map_value), *mutable))
            }
            Self::Call(func, args, _) => {
                codegen.call(map_value(func), args.iter().map(map_value).collect())
            }
            Self::Return(value) => {
                codegen.return_(value.as_ref().map(map_value));
                None
            }

            Self::Intrinsic(intrinsic) => {
                use oc::Intrinsics as _;
                let mut ci = codegen.intrinsics();
                Some(match intrinsic {
                    I::Add(a, b) => ci.add(map_value(a), map_value(b)),
                    I::Mul(a, b) => ci.mul(map_value(a), map_value(b)),
                })
            }

            Self::ACFJump(label) => {
                codegen.acf().jump(map_label(*label));
                None
            }
            Self::ACFCJump(value, label) => {
                codegen.acf().cjump(map_value(value), map_label(*label));
                None
            }
        }
    }
}
