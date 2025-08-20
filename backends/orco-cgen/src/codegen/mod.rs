use crate::{Backend, ob, tm};

impl ob::DefinitionBackend for Backend {
    fn define_function(&mut self, name: ob::Symbol) -> impl ob::Codegen<'_> {
        let function = self.function_decls[&name].clone();
        Codegen {
            backend: self,
            function,
            next_value_id: 0,
            next_label_id: 0,
        }
    }
}

pub struct Codegen<'a> {
    backend: &'a mut Backend,
    function: tm::Function,
    next_value_id: usize,
    next_label_id: usize,
}

impl Codegen<'_> {
    fn new_value(&mut self, expr: tm::Expr, mut ty: tm::Type) -> ob::Value {
        use ob::Codegen as _;
        let symbol = self.new_slot();

        ty.qualifiers.insert(0, tm::TypeQualifier::Const);
        let decl = tm::Variable::new(symbol.to_string(), ty)
            .value(expr)
            .build();
        self.function.body.stmts.push(tm::Statement::Variable(decl));

        ob::Value(symbol.to_ffi() as _)
    }

    fn use_value(&self, value: ob::Value) -> tm::Expr {
        tm::Expr::new_ident_with_str(
            ob::Symbol::try_from_ffi(value.0 as _)
                .unwrap_or_else(|| panic!("invalid value {:?}", value))
                .as_str(),
        )
    }
}

impl<'a> ob::Codegen<'a> for Codegen<'a> {
    type PTS = Backend;

    fn pts(&self) -> &Self::PTS {
        &self.backend
    }

    fn param(&self, idx: usize) -> ob::Symbol {
        self.function.params[idx].name.as_str().into()
    }

    fn iconst(&mut self, ty: ob::Type, value: i128) -> ob::Value {
        // TODO: Size literals
        self.new_value(
            tm::Expr::Int(value as _),
            self.backend.convert_type(&ty).build(),
        )
    }

    fn uconst(&mut self, ty: ob::Type, value: u128) -> ob::Value {
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
        _value: Option<ob::Value>,
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

    fn variable(&mut self, symbol: ob::Symbol) -> ob::Value {
        ob::Value(symbol.to_ffi() as _)
    }

    fn new_slot(&mut self) -> ob::Symbol {
        let symbol = ob::Symbol::new(format!("__v{}", self.next_value_id));
        self.next_value_id += 1;
        symbol
    }

    fn return_(&mut self, value: Option<ob::Value>) {
        self.function.body.stmts.push(tm::Statement::Return(
            value.map(|value| self.use_value(value)),
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
