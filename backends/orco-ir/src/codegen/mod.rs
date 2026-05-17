use crate::ir;
use orco::codegen as oc;

mod intrinsics;

/// Implementation of [`oc::BodyCodegen`]
pub struct Codegen<'a, 'b: 'a> {
    /// Backend context that will recieve the symbol once codegen is done
    pub backend: &'a crate::Backend<'b>,
    /// Symbol name
    pub name: orco::Symbol,
    /// Currently generated body
    pub body: ir::Body,
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
        }
    }

    fn stmt(&mut self, statement: ir::Statement) -> oc::Value {
        self.body.statements.push(statement);
        oc::Value(self.body.statements.len() - 1)
    }
}

impl oc::BodyCodegen for Codegen<'_, '_> {
    fn comment(&mut self, comment: &str) {
        self.stmt(ir::Statement::Comment(comment.to_owned()));
    }

    fn type_of(&self, id: usize) -> orco::Type {
        self.body
            .statements
            .get(id)
            .unwrap_or_else(|| panic!("invalid value id {id}"))
            .get_type(self.backend, &self.body)
    }

    fn declare_var(&mut self, ty: orco::Type) -> oc::Variable {
        self.body.variables.push(ir::Variable { ty, arg: false });
        oc::Variable(self.body.variables.len() - 1)
    }

    fn assign(&mut self, target: oc::Place, value: oc::Value) {
        self.stmt(ir::Statement::Assign(target, value));
    }

    fn iconst(&mut self, value: i128, size: orco::types::IntegerSize) -> oc::Value {
        self.stmt(ir::Statement::IConst(value, size))
    }

    fn uconst(&mut self, value: u128, size: orco::types::IntegerSize) -> oc::Value {
        self.stmt(ir::Statement::UConst(value, size))
    }

    fn fconst(&mut self, value: f64, size: u16) -> oc::Value {
        self.stmt(ir::Statement::FConst(value, size))
    }

    fn bconst(&mut self, value: bool) -> oc::Value {
        self.stmt(ir::Statement::BConst(value))
    }

    fn read(&mut self, place: oc::Place) -> oc::Value {
        self.stmt(ir::Statement::Read(place))
    }

    fn reference(&mut self, place: oc::Place, mutable: bool) -> oc::Value {
        let can_be_mutable = ir::place_ty(&place, self.backend, &self.body).1;
        if mutable && !can_be_mutable {
            panic!("can't create mutable reference to an immutable {place}")
        }

        self.stmt(ir::Statement::Reference(place, mutable))
    }

    fn call(&mut self, func: oc::Value, args: Vec<oc::Value>) -> Option<oc::Value> {
        let has_retval = match self.type_of(func.0) {
            orco::Type::FnPtr { return_type, .. } => return_type.is_some(),
            ty => panic!("trying to call non-function {func}, which is of type {ty}"),
        };
        has_retval.then_some(self.stmt(ir::Statement::Call(func, args, has_retval)))
    }

    fn return_(&mut self, value: Option<oc::Value>) {
        self.stmt(ir::Statement::Return(value));
    }

    fn intrinsics(&mut self) -> impl oc::Intrinsics + '_ {
        self
    }

    fn acf(&mut self) -> impl oc::ACFCodegen + '_ {
        self
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
