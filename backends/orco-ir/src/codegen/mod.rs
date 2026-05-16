use crate::ir;
use orco::codegen as oc;
use std::collections::HashMap;

/// Implementation of [`oc::BodyCodegen`]
pub struct Codegen<'a, 'b: 'a> {
    /// Backend context that will recieve the symbol once codegen is done
    pub backend: &'a crate::Backend<'b>,
    /// Symbol name
    pub name: orco::Symbol,
    /// Currently generated body
    pub body: ir::Body,
    /// Map of [`oc::Value::0`] to value type. Entries get
    /// removed whenever values get used
    values: HashMap<usize, orco::Type>,
}

impl<'a, 'b: 'a> Codegen<'a, 'b> {
    #[allow(missing_docs)]
    pub fn new(backend: &'a crate::Backend<'b>, name: orco::Symbol) -> Self {
        let mut body = ir::Body::default();
        let function = backend
            .functions
            .get_sync(&name)
            .unwrap_or_else(|| panic!("trying to codegen undeclared function {name}"));

        body.variables.reserve(function.params.len());
        for (_, ty) in &function.params {
            body.variables.push(ir::Variable {
                ty: ty.clone(),
                arg: true,
            });
        }

        Self {
            backend,
            name,
            body,
            values: HashMap::new(),
        }
    }

    fn stmt(&mut self, statement: ir::Statement) {
        dbg!(&statement);
        self.body.statements.push(statement);
    }

    /// Generate value from last inserted statement
    /// with set type
    fn value(&mut self, ty: orco::Type) -> oc::Value {
        let id = self.body.statements.len() - 1;
        self.values.insert(id, ty);
        oc::Value(id)
    }
}

impl oc::BodyCodegen for Codegen<'_, '_> {
    fn comment(&mut self, comment: &str) {
        self.stmt(ir::Statement::Comment(comment.to_owned()))
    }

    fn type_of(&self, id: usize) -> orco::Type {
        self.values.get(&id).unwrap_or_else(|| panic!("trying to determine type from invalid value id {id}. Something probably went terribly wrong")).clone()
    }

    fn declare_var(&mut self, ty: orco::Type) -> oc::Variable {
        self.body.variables.push(ir::Variable { ty, arg: false });
        oc::Variable(self.body.variables.len() - 1)
    }

    fn assign(&mut self, target: oc::Place, value: oc::Value) {
        self.stmt(ir::Statement::Assign(target, value));
    }

    fn iconst(&mut self, value: i128, size: orco::types::IntegerSize) -> oc::Value {
        self.stmt(ir::Statement::IConst(value, size));
        self.value(orco::Type::Integer(size))
    }

    fn uconst(&mut self, value: u128, size: orco::types::IntegerSize) -> oc::Value {
        self.stmt(ir::Statement::UConst(value, size));
        self.value(orco::Type::Unsigned(size))
    }

    fn fconst(&mut self, value: f64, size: u16) -> oc::Value {
        self.stmt(ir::Statement::FConst(value, size));
        self.value(orco::Type::Float(size))
    }

    fn bconst(&mut self, value: bool) -> oc::Value {
        self.stmt(ir::Statement::BConst(value));
        self.value(orco::Type::Bool)
    }

    fn read(&mut self, place: oc::Place) -> oc::Value {
        self.stmt(ir::Statement::Comment("".to_owned()));
        self.value(orco::Type::Error)
    }

    fn reference(&mut self, place: oc::Place) -> oc::Value {
        self.stmt(ir::Statement::Comment("".to_owned()));
        self.value(orco::Type::Error)
    }

    fn call(&mut self, func: oc::Value, args: Vec<oc::Value>) -> Option<oc::Value> {
        None
    }

    fn return_(&mut self, value: Option<oc::Value>) {}

    fn intrinsics(&mut self) -> impl oc::Intrinsics + '_ {
        self
    }

    fn acf(&mut self) -> impl oc::ACFCodegen + '_ {
        self
    }
}

impl oc::Intrinsics for &mut Codegen<'_, '_> {
    fn add(&mut self, a: oc::Value, b: oc::Value) -> oc::Value {
        self.stmt(ir::Statement::Comment("".to_owned()));
        self.value(orco::Type::Error)
    }

    fn mul(&mut self, a: oc::Value, b: oc::Value) -> oc::Value {
        self.stmt(ir::Statement::Comment("".to_owned()));
        self.value(orco::Type::Error)
    }
}

impl oc::ACFCodegen for &mut Codegen<'_, '_> {
    fn alloc_label(&mut self) -> oc::Label {
        oc::Label(0)
    }

    fn label(&mut self, label: oc::Label) {}

    fn jump(&mut self, label: oc::Label) {}

    fn cjump(&mut self, condition: oc::Value, label: oc::Label) {}
}

impl std::ops::Drop for Codegen<'_, '_> {
    fn drop(&mut self) {
        self.backend
            .function_definitions
            .insert_sync(self.name, std::mem::take(&mut self.body))
            .unwrap_or_else(|_| panic!("function {} is already defined", self.name));
    }
}
