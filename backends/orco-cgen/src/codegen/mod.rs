use crate::{Backend, ob, tm};

impl ob::DefinitionBackend for Backend {
    type Codegen<'a> = Codegen<'a>;

    fn define_function(&mut self, name: ob::Symbol) -> Self::Codegen<'_> {
        let function = self.function_decls[&name].clone();
        Codegen {
            backend: self,
            function,
        }
    }
}

pub struct Codegen<'a> {
    backend: &'a mut Backend,
    function: tm::Function,
}

impl<'a> ob::Codegen<'a> for Codegen<'a> {
    type PTS = Backend;
    type Value = tm::Expr;

    fn pts(&self) -> &Self::PTS {
        &self.backend
    }

    fn iconst(&mut self, _ty: ob::Type, value: i128) -> Self::Value {
        // TODO: Use size
        tm::Expr::Int(value as _)
    }

    fn uconst(&mut self, _ty: ob::Type, value: u128) -> Self::Value {
        // TODO: Use size
        tm::Expr::UInt(value as _)
    }

    fn return_(&mut self, value: Option<Self::Value>) {
        self.function.body.stmts.push(tm::Statement::Return(value));
    }
}

impl Drop for Codegen<'_> {
    fn drop(&mut self) {
        if self.function.body.stmts.is_empty() {
            self.function.body.stmts.push(tamago::Statement::NewLine);
        }
        self.backend.function_defs.push(std::mem::replace(
            &mut self.function,
            tm::Function::new(String::new(), tm::Type::new(tm::BaseType::Void).build()).build(),
        ));
    }
}
