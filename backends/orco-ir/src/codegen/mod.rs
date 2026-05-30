use crate::ir;
use orco::codegen as oc;
use std::collections::HashMap;

mod intrinsics;

/// Implementation of [`oc::BodyCodegen`]
pub struct Codegen<'a, 'b: 'a> {
    /// Backend context that will recieve the symbol once codegen is done
    pub backend: &'a crate::Backend<'b>,
    /// Symbol name
    pub name: orco::Symbol,
    /// Currently generated body
    pub body: ir::Body,
    /// Currently unused values
    values: HashMap<usize, ir::Expression>,
    /// Next value ID
    next_value_id: usize,
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
        for (name, ty) in &function.params {
            body.variables.push(ir::Variable {
                ty: ty.clone(),
                arg: true,
                name: name.clone(),
            });
        }

        Self {
            backend,
            name,
            body,
            values: HashMap::new(),
            next_value_id: 0,
        }
    }

    /// Insert an expression and return [`oc::Value`] for it
    pub fn expr(&mut self, expr: ir::Expression) -> oc::Value {
        let id = self.next_value_id;
        self.next_value_id += 1;
        self.values.insert(id, expr);
        oc::Value(id)
    }

    /// Convert [`oc::Value`] back to an expression, taking it out.
    /// Opposite to [`Self::expr`]
    pub fn use_value(&mut self, value: oc::Value) -> ir::Expression {
        self.values
            .remove(&value.0)
            .unwrap_or_else(|| panic!("invalid or previously used value #{}", value.0))
    }

    /// Convert [`oc::Place`] to [`ir::Place`]
    pub fn cvt_place(&mut self, place: oc::Place) -> ir::Place {
        match place {
            oc::Place::Variable(variable) => ir::Place::Variable(variable),
            oc::Place::Global(name) => ir::Place::Global(name),
            oc::Place::Deref(value) => ir::Place::Deref(Box::new(self.use_value(value))),
            oc::Place::Field(place, idx) => ir::Place::Field(Box::new(self.cvt_place(*place)), idx),
        }
    }
}

impl oc::BodyCodegen for Codegen<'_, '_> {
    fn comment(&mut self, comment: &str) {
        self.body
            .statements
            .push(ir::Statement::Comment(comment.to_owned()));
    }

    fn type_of(&self, id: usize) -> orco::Type {
        self.values
            .get(&id)
            .unwrap_or_else(|| panic!("invalid value id {id}"))
            .get_type(self.backend, &self.body)
    }

    fn declare_var(&mut self, ty: orco::Type, name: Option<&str>) -> oc::Variable {
        self.body.variables.push(ir::Variable {
            ty,
            arg: false,
            name: name.map(|name| name.to_owned()),
        });
        oc::Variable(self.body.variables.len() - 1)
    }

    fn assign(&mut self, target: oc::Place, value: oc::Value) {
        let target = self.cvt_place(target);
        let value = self.use_value(value);
        self.body
            .statements
            .push(ir::Statement::Assign(target, value))
    }

    fn iconst(&mut self, value: i128, size: orco::types::IntegerSize) -> oc::Value {
        self.expr(ir::Expression::IConst(value, size))
    }

    fn uconst(&mut self, value: u128, size: orco::types::IntegerSize) -> oc::Value {
        self.expr(ir::Expression::UConst(value, size))
    }

    fn fconst(&mut self, value: f64, size: u16) -> oc::Value {
        self.expr(ir::Expression::FConst(value, size))
    }

    fn bconst(&mut self, value: bool) -> oc::Value {
        self.expr(ir::Expression::BConst(value))
    }

    fn read(&mut self, place: oc::Place) -> oc::Value {
        let place = self.cvt_place(place);
        self.expr(ir::Expression::Read(place))
    }

    fn reference(&mut self, place: oc::Place, mutable: bool) -> oc::Value {
        let place = self.cvt_place(place);
        let can_be_mutable = place.get_type(self.backend, &self.body).1;
        if mutable && !can_be_mutable {
            panic!("can't create mutable reference to an immutable {place}")
        }

        self.expr(ir::Expression::Reference(place, mutable))
    }

    fn call(&mut self, func: oc::Value, args: Vec<oc::Value>) -> Option<oc::Value> {
        let func = self.use_value(func);
        let has_retval = match func.get_type(self.backend, &self.body) {
            orco::Type::FnPtr { return_type, .. } => return_type.is_some(),
            ty => panic!("trying to call non-function {func}, which is of type {ty}"),
        };

        let args = args.into_iter().map(|arg| self.use_value(arg)).collect();
        if has_retval {
            Some(self.expr(ir::Expression::Call(Box::new(func), args)))
        } else {
            self.body.statements.push(ir::Statement::Call(func, args));
            None
        }
    }

    fn return_(&mut self, value: Option<oc::Value>) {
        let value = value.map(|value| self.use_value(value));
        self.body.statements.push(ir::Statement::Return(value));
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
        self.body.labels.push(0);
        oc::Label(self.body.labels.len() - 1)
    }

    fn label(&mut self, label: oc::Label) {
        self.body.labels[label.0] = self.body.statements.len();
    }

    fn jump(&mut self, label: oc::Label) {
        self.body.statements.push(ir::Statement::ACFJump(label));
    }

    fn cjump(&mut self, condition: oc::Value, label: oc::Label) {
        let condition = self.use_value(condition);
        self.body
            .statements
            .push(ir::Statement::ACFCJump(condition, label));
    }
}

impl std::ops::Drop for Codegen<'_, '_> {
    fn drop(&mut self) {
        self.backend
            .function_definitions
            .insert_sync(self.name, std::mem::take(&mut self.body))
            .unwrap_or_else(|_| panic!("function {} is already defined", self.name));
    }
}
