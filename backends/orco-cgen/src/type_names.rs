impl super::Backend {
    /// If ty is a type alias (but not a struct), inlines it.
    /// Does not inline inner types
    pub fn inline_type_aliases<'a>(
        &self,
        guard: &'a impl papaya::Guard,
        mut ty: &'a orco::Type,
        inline_struct: bool,
    ) -> &'a orco::Type {
        while let orco::Type::Symbol(name, generics) = ty {
            let name = self.generic_name(*name, generics);
            let symbol = self
                .types
                .get(&name, guard)
                .unwrap_or_else(|| panic!("undeclared type {name}"));
            if inline_struct || !matches!(*symbol, orco::Type::Struct { .. }) {
                ty = symbol;
            } else {
                return ty;
            }
        }

        ty
    }

    /// Intern the following type and it's insides.
    /// If `named` contains a value, it's the name of the current typedef
    pub fn intern_type(&self, ty: &mut orco::Type, named: Option<orco::Symbol>) {
        use orco::Type;

        // Intern inner types
        match ty {
            Type::Symbol(name, generics) => {
                for ty in generics.iter_mut() {
                    self.intern_type(ty, None);
                }
                *ty = Type::Symbol(self.generic_name(*name, generics), Vec::new());
            }
            Type::Array(ty, _) => self.intern_type(ty.as_mut(), None),
            Type::Struct { fields } => {
                for (_, ty) in fields {
                    self.intern_type(ty, None)
                }
            }
            Type::Ptr(ty, _) => self.intern_type(ty, None),
            Type::FnPtr {
                params,
                return_type,
            } => {
                for ty in params {
                    self.intern_type(ty, None);
                }
                if let Some(ty) = return_type {
                    self.intern_type(ty, None);
                }
            }
            _ => (),
        }

        // Intern this type (if required)
        match ty {
            Type::Struct { .. } => {
                let interned = self.interned.pin();
                if let Some(name) = interned.get(ty) {
                    *ty = orco::Type::Symbol(*name, Vec::new());
                } else {
                    if let Some(name) = named {
                        interned.insert(ty.clone(), name);
                    } else {
                        use orco::DeclarationBackend as _;
                        let name = self.unified_type_name(ty).into();
                        let ty = core::mem::replace(ty, Type::Symbol(name, Vec::new()));
                        self.type_(name, Vec::new(), ty);
                    }
                }
            }
            _ => (),
        }
    }

    /// "unified type name" is one string that any two compatible types would map to.
    pub fn unified_type_name(&self, ty: &orco::Type) -> String {
        use orco::Type;
        match self.inline_type_aliases(&self.types.guard(), ty, false) {
            Type::Integer(..)
            | Type::Unsigned(..)
            | Type::Float(..)
            | Type::Bool
            | Type::Char(..)
            | Type::Param(..)
            | Type::Error => ty.to_string().into(),
            Type::Symbol(name, generics) => self.generic_name(*name, generics).into(),
            Type::Array(ty, size) => format!("{}[{size}]", self.unified_type_name(ty)).into(),
            Type::Struct { fields } => {
                let mut name = "{".to_owned();
                for (idx, (field, ty)) in fields.iter().enumerate() {
                    if idx > 0 {
                        name.push_str(", ");
                    }
                    if let Some(field) = field {
                        name.push_str(field);
                        name.push(':');
                    }
                    name.push_str(&self.unified_type_name(ty));
                }
                name.push('}');
                name.into()
            }
            Type::Ptr(ty, mutable) => format!(
                "*{} {}",
                match mutable {
                    true => "mut",
                    false => "const",
                },
                self.unified_type_name(ty)
            )
            .into(),
            Type::FnPtr {
                params,
                return_type,
            } => {
                let mut name = "(".to_owned();
                for (idx, param) in params.iter().enumerate() {
                    if idx > 0 {
                        name.push_str(", ");
                    }

                    name.push_str(&self.unified_type_name(param));
                }

                match return_type {
                    Some(ty) => {
                        name.push_str(") -> ");
                        name.push_str(&self.unified_type_name(ty));
                    }
                    None => name.push_str(") -> void"),
                }

                name.into()
            }
        }
    }

    /// Embed generics in the symbol name (also handles disambiguation and interning)
    pub fn generic_name(&self, name: orco::Symbol, generics: &[orco::Type]) -> orco::Symbol {
        if generics.is_empty() {
            return name;
        }

        let mut name = name.to_string();
        name.push('<');
        for (idx, ty) in generics.iter().enumerate() {
            if ty.has_params() {
                panic!("generic params are not supported (encountered {ty})");
            }

            if idx > 0 {
                name.push_str(", ");
            }

            name.push_str(&self.unified_type_name(ty));
        }

        name.push('>');
        name.into()
    }
}
