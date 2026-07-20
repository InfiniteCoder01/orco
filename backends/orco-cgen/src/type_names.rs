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
                        let name = ty.to_string().into();
                        let ty = core::mem::replace(ty, Type::Symbol(name, Vec::new()));
                        self.type_(name, Vec::new(), ty);
                    }
                }
            }
            _ => (),
        }
    }

    /// Embed generics in the symbol name (also handles disambiguation and interning)
    pub fn generic_name(&self, name: orco::Symbol, generics: &[orco::Type]) -> orco::Symbol {
        if generics.is_empty() {
            return name;
        }

        for ty in generics {
            if ty.has_params() {
                panic!("generic params are not supported (encountered {ty})");
            }
        }

        format!("{name}{}", orco::types::fmt_generics(generics)).into()
    }
}
