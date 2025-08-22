use crate::{Backend, ob, tm};

impl ob::DefinitionBackend for Backend {
    fn define_function(&self, name: ob::Symbol) -> impl ob::Codegen<'_> {
        let function = self.function_decls[&name].clone();
        Codegen {
            backend: self,
            function,
            blocks: Vec::new(),
            next_value_id: 0,
            _next_label_id: 0,
        }
    }
}

/// A block where current codegen takes place
enum Block {
    If(tm::If),
}

pub struct Codegen<'a> {
    backend: &'a Backend,
    function: tm::Function,
    blocks: Vec<Block>,
    next_value_id: usize,
    _next_label_id: usize,
}

impl Codegen<'_> {
    fn stmt(&mut self, stmt: tm::Statement) {
        let block = self
            .blocks
            .last_mut()
            .map_or(&mut self.function.body, |block| match block {
                Block::If(if_) => if_.other.as_mut().unwrap_or(&mut if_.then),
            });
        block.stmts.push(stmt);
    }

    fn new_value(&mut self, expr: tm::Expr, mut ty: tm::Type) -> ob::Value {
        use ob::Codegen as _;
        let symbol = self.new_slot();

        ty.qualifiers.insert(0, tm::TypeQualifier::Const);
        let decl = tm::Variable::new(symbol.to_string(), ty)
            .value(expr)
            .build();
        self.stmt(tm::Statement::Variable(decl));

        ob::Value(symbol.to_ffi() as _)
    }

    fn use_value(&self, value: ob::Value) -> tm::Expr {
        tm::Expr::new_ident_with_str(
            ob::Symbol::try_from_ffi(value.0 as _)
                .unwrap_or_else(|| panic!("invalid value {value:?}"))
                .as_str(),
        )
    }
}

impl<'a> ob::Codegen<'a> for Codegen<'a> {
    fn backend(&self) -> &impl ob::DefinitionBackend {
        self.backend
    }

    fn param(&self, idx: usize) -> ob::Symbol {
        self.function.params[idx].name.as_str().into()
    }

    fn iconst(&mut self, _ty: ob::Type, value: i128) -> ob::Value {
        // TODO: Size literals
        self.variable(value.to_string().into())
    }

    fn uconst(&mut self, _ty: ob::Type, value: u128) -> ob::Value {
        // TODO: Size literals
        self.variable(value.to_string().into())
    }

    fn define_variable(
        &mut self,
        name: ob::Symbol,
        ty: ob::Type,
        mutable: bool,
        value: Option<ob::Value>,
    ) {
        let mut ty = self.backend.convert_type(&ty).build();
        if !mutable {
            ty.qualifiers.push(tm::TypeQualifier::Const);
        }

        let mut decl = tm::Variable::new(name.to_string(), ty).build();
        decl.value = value.map(|value| self.use_value(value));
        self.stmt(tm::Statement::Variable(decl));
    }

    fn assign_variable(&mut self, name: ob::Symbol, value: ob::Value) {
        self.stmt(tm::Statement::Expr(tm::Expr::new_assign(
            tm::Expr::new_ident_with_str(name.as_str()),
            tm::AssignOp::Assign,
            self.use_value(value),
        )));
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
        self.stmt(tm::Statement::Return(
            value.map(|value| self.use_value(value)),
        ));
    }

    fn if_(&mut self, cond: ob::Value) {
        self.blocks
            .push(Block::If(tm::If::new(self.use_value(cond)).build()));
    }

    fn else_(&mut self) {
        match self.blocks.last_mut() {
            Some(Block::If(if_)) => {
                if_.other = Some(tm::Block::new().build());
            }
            _ => panic!("tried to add else to a different kind of block"),
        }
    }

    fn end(&mut self) {
        match self.blocks.pop() {
            Some(Block::If(if_)) => {
                self.stmt(tm::Statement::If(if_));
            }
            None => panic!("tried to use end without a block"),
        }
    }
}

impl Drop for Codegen<'_> {
    fn drop(&mut self) {
        if self.function.body.stmts.is_empty() {
            self.function.body.stmts.push(tamago::Statement::NewLine);
        }
        self.backend
            .function_defs
            .lock()
            .unwrap()
            .push(std::mem::replace(
                &mut self.function,
                tm::Function::new(String::new(), tm::Type::new(tm::BaseType::Void).build()).build(),
            ));
    }
}
