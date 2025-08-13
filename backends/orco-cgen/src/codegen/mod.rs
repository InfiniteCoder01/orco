use crate::{Backend, ob, tm};

impl ob::DefinitionBackend for Backend {
    type FunctionCodegen<'a> = Codegen<'a>;

    fn define_function(&mut self, name: ob::Symbol) -> Self::FunctionCodegen<'_> {
        let function = self.function_decls[&name].clone();
        Codegen {
            backend: self,
            function,
            next_value_id: 0,
        }
    }
}

pub struct Codegen<'a> {
    backend: &'a mut Backend,
    function: tm::Function,
    next_value_id: usize,
}

impl Codegen<'_> {
    fn new_value(&mut self, value: tm::Expr, mut ty: tm::Type) -> ob::Symbol {
        let name = ob::Symbol::new(format!("__v{}", self.next_value_id));
        ty.qualifiers.insert(0, tm::TypeQualifier::Const);

        let decl = tm::Variable::new(name.to_string(), ty).value(value).build();
        self.function.body.stmts.push(tm::Statement::Variable(decl));
        self.next_value_id += 1;
        return name;
    }
}

impl<'a> ob::FunctionCodegen<'a> for Codegen<'a> {
    type PTS = Backend;
    type Value = ob::Symbol; // Values are just variables here

    fn pts(&self) -> &Self::PTS {
        &self.backend
    }

    fn param(&self, idx: usize) -> ob::Symbol {
        self.function.params[idx].name.as_str().into()
    }

    fn iconst(&mut self, ty: ob::Type, value: i128) -> Self::Value {
        // TODO: Size literals
        self.new_value(
            tm::Expr::Int(value as _),
            self.backend.convert_type(&ty).build(),
        )
    }

    fn uconst(&mut self, ty: ob::Type, value: u128) -> Self::Value {
        // TODO: Size literals
        self.new_value(
            tm::Expr::UInt(value as _),
            self.backend.convert_type(&ty).build(),
        )
    }

    fn define_variable(
        &mut self,
        name: ob::Symbol,
        ty: ob::Type,
        mutable: bool,
        value: Option<Self::Value>,
    ) {
        let mut ty = self.backend.convert_type(&ty).build();
        if !mutable {
            ty.qualifiers.push(tm::TypeQualifier::Const);
        }

        let mut decl = tm::Variable::new(name.to_string(), ty).build();
        // decl.value = value;
        self.function.body.stmts.push(tm::Statement::Variable(decl));
        todo!()
    }

    fn variable(&mut self, symbol: ob::Symbol) -> Self::Value {
        symbol
    }

    fn return_(&mut self, value: Option<Self::Value>) {
        self.function.body.stmts.push(tm::Statement::Return(
            value.map(|value| tm::Expr::new_ident_with_str(value.as_str())),
        ));
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
