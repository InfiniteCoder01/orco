use crate::{Backend, ob, tm};

pub mod primitives;

impl ob::DeclarationBackend for Backend {
    fn function(
        &mut self,
        name: ob::Symbol,
        params: &[(Option<ob::Symbol>, ob::Type)],
        return_type: &ob::Type,
    ) {
        let mut function =
            tm::Function::new(name.to_string(), self.build_type(return_type)).build();
        for (name, ty) in params {
            function.params.push(
                tm::Parameter::new(
                    name.map_or("", |name| name.as_str()).to_string(),
                    self.build_type(ty),
                )
                .build(),
            );
        }

        self.function_decls.insert(name, function);
    }
}

impl Backend {
    pub fn convert_type(&self, ty: &ob::Type) -> tm::TypeBuilder {
        match ty {
            ob::Type::Symbol(symbol) => {
                tm::Type::new(tamago::BaseType::TypeDef(symbol.as_str().to_owned()))
            }
        }
    }

    pub fn build_type(&self, ty: &ob::Type) -> tm::Type {
        self.convert_type(ty).build()
    }
}
