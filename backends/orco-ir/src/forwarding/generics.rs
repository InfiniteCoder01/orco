impl crate::Store {
    /// Declare all symbols from this IR in another [`orco::DeclarationBackend`],
    /// monomorphizing generics
    pub fn declare_mono(&self, backend: &impl orco::DeclarationBackend) {
        let type_instances = self.type_instances.pin();
        for (name, specs) in self.types.pin().iter() {
            for (generics, ty) in specs.pin().iter() {
                if generics.iter().any(orco::Type::has_params) {
                    continue;
                }

                if type_instances.insert((*name, generics.clone())) {
                    self.register_type(ty, backend);
                    backend.type_(*name, generics.clone(), ty.clone());
                }
            }
        }

        // for (name, decl) in self.functions.pin().iter() {
        //     backend.function(
        //         *name,
        //         decl.generic_params.clone(),
        //         decl.signature.params.clone(),
        //         decl.signature.return_type.clone(),
        //         decl.signature.attrs.clone(),
        //     );
        // }
    }

    /// Register a type instance for monomorphization, see [`Self::type_instances`]
    pub fn register_type(&self, ty: &orco::Type, backend: &impl orco::DeclarationBackend) {
        use orco::Type;
        match ty {
            Type::Integer(..)
            | Type::Unsigned(..)
            | Type::Float(..)
            | Type::Bool
            | Type::Char(..) => (),
            Type::Symbol(name, generics) => {
                if self.type_instances.pin().insert((*name, generics.clone())) {
                    self.get_type(*name, generics, |ty, map| {
                        let ty = ty.copy_instantiate(&map);
                        self.register_type(&ty, backend);
                        backend.type_(*name, generics.clone(), ty)
                    });
                }
            }
            Type::Array(ty, _) => self.register_type(ty, backend),
            Type::Struct { fields } => {
                for (_, ty) in fields {
                    self.register_type(ty, backend);
                }
            }
            Type::Ptr(ty, _) => self.register_type(ty, backend),
            Type::FnPtr {
                params,
                return_type,
            } => {
                for ty in params {
                    self.register_type(ty, backend);
                }
                if let Some(ty) = return_type {
                    self.register_type(ty, backend);
                }
            }
            Type::Param(name) => {
                panic!("encountered a type param #{name} while recording type instances")
            }
            Type::Error => (),
        }
    }
}
