impl crate::Store {
    /// Generate monomorphization, see [`crate::Store::type_instances`]
    /// and [`crate::Store::function_instances`]
    pub fn monomorphize(&self) {
        let type_instances = self.type_instances.pin();
        let function_instances = self.type_instances.pin();
        type_instances.clear();
        function_instances.clear();

        for (name, specs) in self.types.pin().iter() {
            for (generics, ty) in specs.pin().iter() {
                if generics.iter().any(orco::Type::has_params) {
                    continue;
                }

                if self.type_instances.pin().insert((*name, generics.clone())) {
                    self.register_type(&ty);
                }
            }
        }

        let bodies = self.function_bodies.pin();
        for (name, decl) in self.functions.pin().iter() {
            if !decl.generic_params.iter().any(orco::Type::has_params) {
                self.register_funcion(*name, &decl.generic_params);
                continue;
            }

            let Some(specs) = bodies.get(name) else {
                continue;
            };

            for (generics, _) in specs.pin().iter() {
                if !decl.generic_params.iter().any(orco::Type::has_params) {
                    self.register_funcion(*name, generics);
                }
            }
        }
    }

    /// Declare all symbols from this IR in another [`orco::DeclarationBackend`],
    /// monomorphizing generics
    pub fn declare_mono(&self, backend: &impl orco::DeclarationBackend) {
        for (name, generics) in self.type_instances.pin().iter() {
            self.get_type(*name, generics, |ty, map| {
                backend.type_(*name, generics.clone(), ty.copy_instantiate(&map));
            });
        }

        let functions = self.functions.pin();
        for (name, generics) in self.function_instances.pin().iter() {
            let decl = functions
                .get(name)
                .unwrap_or_else(|| panic!("function {name} not found"));
            let sig = decl.instantiate(self, generics);
            backend.function(
                *name,
                generics.clone(),
                sig.params.clone(),
                sig.return_type.clone(),
                sig.attrs.clone(),
            );
        }
    }

    /// Register a type instance for monomorphization, see [`Self::type_instances`]
    pub fn register_type(&self, ty: &orco::Type) {
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
                        self.register_type(&ty);
                    });
                }
            }
            Type::Array(ty, _) => self.register_type(ty),
            Type::Struct { fields } => {
                for (_, ty) in fields {
                    self.register_type(ty);
                }
            }
            Type::Ptr(ty, _) => self.register_type(ty),
            Type::FnPtr {
                params,
                return_type,
            } => {
                for ty in params {
                    self.register_type(ty);
                }
                if let Some(ty) = return_type {
                    self.register_type(ty);
                }
            }
            Type::Param(name) => {
                panic!("encountered a type param #{name} while recording type instances")
            }
            Type::Error => (),
        }
    }

    /// Register a type instance for monomorphization, see [`Self::type_instances`]
    pub fn register_funcion(&self, name: orco::Symbol, generics: &[orco::Type]) {
        self.function_instances
            .pin()
            .insert((name, generics.to_vec()));
        let functions = self.functions.pin();
        let decl = functions
            .get(&name)
            .unwrap_or_else(|| panic!("function {name} not found"));

        let signature = decl.instantiate(self, generics);
        for (_, ty) in &signature.params {
            self.register_type(ty);
        }
        if let Some(ty) = &signature.return_type {
            self.register_type(&ty);
        }

        let bodies = self.function_bodies.pin();
        let Some(specs) = bodies.get(&name) else {
            return;
        };
        crate::generics::match_specialization(&specs, generics, self, |body, map| {
            for variable in &body.variables {
                self.register_type(&variable.ty.copy_instantiate(&map));
            }

            use crate::ir::Expression;
            use crate::ir::Place;
            use crate::ir::Statement;
            fn register_place(store: &crate::Store, place: &Place) {
                match place {
                    Place::Variable(..) => todo!(),
                    Place::Global(name, generics) => {
                        store.register_funcion(*name, generics);
                    }
                    Place::Deref(expression) => register_expression(store, expression),
                    Place::Field(place, _) => register_place(store, place),
                }
            }

            fn register_expression(store: &crate::Store, expression: &Expression) {
                match expression {
                    Expression::IConst(..)
                    | Expression::UConst(..)
                    | Expression::FConst(..)
                    | Expression::BConst(..) => (),
                    Expression::Read(place) => register_place(store, place),
                    Expression::Reference(place, _) => register_place(store, place),
                    Expression::Call(function, args) => {
                        register_expression(store, function);
                        for arg in args {
                            register_expression(store, arg);
                        }
                    }
                    Expression::Intrinsic(intrinsic) => {
                        use crate::ir::Intrinsic;
                        match intrinsic {
                            Intrinsic::Add(a, b) | Intrinsic::Mul(a, b) | Intrinsic::Eq(a, b) => {
                                register_expression(store, a);
                                register_expression(store, b);
                            }
                            Intrinsic::Not(value) => {
                                register_expression(store, value);
                            }
                        }
                    }
                }
            }

            for stmt in &body.statements {
                match stmt {
                    Statement::Comment(..) => (),
                    Statement::Assign(place, expression) => {
                        register_place(self, place);
                        register_expression(self, expression);
                    }
                    Statement::Call(function, args) => {
                        register_expression(self, function);
                        for arg in args {
                            register_expression(self, arg);
                        }
                    }
                    Statement::Return(retval) => {
                        if let Some(expr) = retval {
                            register_expression(self, expr);
                        }
                    }
                    Statement::Acf(statement) => {
                        use crate::ir::AcfStatement;
                        match statement {
                            AcfStatement::Jump(..) => (),
                            AcfStatement::Cjump(expression, _) => {
                                register_expression(self, expression)
                            }
                        }
                    }
                    Statement::Bcf(statement) => {
                        use crate::ir::BcfStatement;
                        match statement {
                            BcfStatement::Else
                            | BcfStatement::End
                            | BcfStatement::Loop
                            | BcfStatement::Break
                            | BcfStatement::Continue => (),
                            BcfStatement::If(expression)
                            | BcfStatement::Cbreak(expression)
                            | BcfStatement::Ccontinue(expression) => {
                                register_expression(self, expression)
                            }
                        }
                    }
                }
            }
        });
    }
}
