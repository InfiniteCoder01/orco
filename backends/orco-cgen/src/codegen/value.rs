use super::oc;

pub(super) struct ValueInfo {
    /// Expression for this value, will either be placed whenever
    /// the value is used or placed in the code whenever the value is flushed.
    pub(super) expression: String,
    pub(super) ty: orco::Type,
}

impl ValueInfo {
    pub(super) fn new(expression: String, ty: orco::Type) -> Self {
        Self { expression, ty }
    }
}

impl super::Codegen<'_> {
    /// Make a value and put it in [`Self::values`]
    pub(super) fn mk_value(&mut self, value: ValueInfo) -> oc::Value {
        let id = oc::Value(self.next_value_id);
        self.next_value_id += 1;
        self.values.insert(id.0, value);
        id
    }

    /// Use a value, returns it's expression, must be placed in the code to avoid
    /// missing side effects.
    pub(super) fn use_value(&mut self, value: oc::Value) -> ValueInfo {
        self.values.remove(&value.0).unwrap_or_else(|| {
            panic!(
                "value #{} either doesn't exist or was used already",
                value.0
            )
        })
    }

    /// Convert [`oc::Place`] to a value (C code string + type)
    pub(super) fn place(&mut self, place: oc::Place) -> ValueInfo {
        match place {
            oc::Place::Variable(variable) => {
                let variable = &self.variables[variable.0];
                ValueInfo::new(variable.name.clone(), variable.ty.clone())
            }
            oc::Place::Global(name) => ValueInfo::new(
                crate::symname(name),
                if let Some(signature) = self.backend.functions.get_sync(&name) {
                    signature.ptr_type()
                } else {
                    panic!("undeclared symbol {name}")
                },
            ),
            oc::Place::Deref(value) => {
                let value = self.use_value(value);
                ValueInfo::new(
                    format!("(*{})", value.expression),
                    match self.backend.inline_type_aliases(value.ty, false) {
                        orco::Type::Ptr(ty, _) => *ty,
                        ty => panic!("trying to dereference a non-pointer type {ty:#?}"),
                    },
                )
            }
            oc::Place::Field(place, idx) => {
                let place = self.place(*place);
                let mut fields = match self.backend.inline_type_aliases(place.ty.clone(), true) {
                    orco::Type::Struct { fields } => fields,
                    ty => panic!("trying to access field #{idx} on a non-struct type {ty:#?}"),
                };
                let (name, ty) = fields.swap_remove(idx);
                let name = name.unwrap_or_else(|| format!("_{idx}"));
                ValueInfo::new(format!("{}.{name}", place.expression), ty)
            }
        }
    }
}
