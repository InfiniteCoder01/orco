use crate::{Backend, ob, tm};

impl ob::DefinitionBackend for Backend {
    type FunctionCodegen<'a> = Function<'a>;

    fn function(&mut self, name: ob::Symbol) -> Self::FunctionCodegen<'_> {
        let function = self.function_decls[&name].clone();
        Function {
            backend: self,
            function,
        }
    }
}

pub struct Function<'a> {
    backend: &'a mut Backend,
    function: tm::Function,
}

impl<'a> ob::FunctionCodegen<'a> for Function<'a> {}

impl Drop for Function<'_> {
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
