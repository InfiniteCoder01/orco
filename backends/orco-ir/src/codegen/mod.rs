use crate::ir;
use orco::codegen as oc;
use std::collections::HashMap;

mod control_flow;
mod intrinsics;

/// Implementation of [`oc::BodyCodegen`]
pub struct Codegen<'a> {
    /// Store that will recieve the symbol once codegen is done
    pub store: &'a crate::Store,
    /// Symbol name
    pub name: orco::Symbol,
    /// Generic params
    pub generic_params: Vec<orco::Type>,
    /// Currently generated body
    pub body: ir::Body,
    /// Currently unused values
    values: HashMap<usize, ir::Expression>,
    /// Next value ID
    next_value_id: usize,
}

impl<'a> Codegen<'a> {
    #[allow(missing_docs)]
    pub fn new(
        store: &'a crate::Store,
        name: orco::Symbol,
        generic_params: Vec<orco::Type>,
    ) -> Self {
        let mut body = ir::Body::default();
        let decls = store.functions.pin();
        let function = decls
            .get(&name)
            .unwrap_or_else(|| panic!("trying to codegen undeclared function {name}"));

        body.variables.reserve(function.signature.params.len());
        for (name, ty) in &function.signature.params {
            body.variables.push(ir::Variable {
                ty: ty.clone(),
                arg: true,
                name: name.clone(),
            });
        }

        Self {
            store,
            name,
            generic_params,
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
            oc::Place::Global(name, generics) => ir::Place::Global(name, generics),
            oc::Place::Deref(value) => ir::Place::Deref(Box::new(self.use_value(value))),
            oc::Place::Field(place, idx) => ir::Place::Field(Box::new(self.cvt_place(*place)), idx),
        }
    }
}

impl oc::BodyCodegen for Codegen<'_> {
    fn comment(&mut self, comment: &str) {
        self.body
            .statements
            .push(ir::Statement::Comment(comment.to_owned()));
    }

    fn type_of(&self, id: usize) -> orco::Type {
        self.values
            .get(&id)
            .unwrap_or_else(|| panic!("invalid value id {id}"))
            .get_type(self.store, &self.body)
    }

    fn declare_var(&mut self, ty: orco::Type, name: Option<&str>) -> oc::Variable {
        self.body.variables.push(ir::Variable {
            ty,
            arg: false,
            name: name.map(std::borrow::ToOwned::to_owned),
        });
        oc::Variable(self.body.variables.len() - 1)
    }

    fn assign(&mut self, target: oc::Place, value: oc::Value) {
        let target = self.cvt_place(target);
        let value = self.use_value(value);
        self.body
            .statements
            .push(ir::Statement::Assign(target, value));
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
        let can_be_mutable = place.get_type(self.store, &self.body).1;
        assert!(
            !mutable || can_be_mutable,
            "can't create mutable reference to an immutable {place}"
        );

        self.expr(ir::Expression::Reference(place, mutable))
    }

    fn call(&mut self, func: oc::Value, args: Vec<oc::Value>) -> Option<oc::Value> {
        let func = self.use_value(func);
        let has_retval = match func.get_type(self.store, &self.body) {
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

    fn acf(&mut self) -> impl oc::AcfCodegen + '_ {
        self
    }

    fn bcf(&mut self) -> impl oc::BcfCodegen + '_ {
        self
    }
}

impl core::ops::Drop for Codegen<'_> {
    fn drop(&mut self) {
        self.store
            .function_bodies
            .pin()
            .get_or_insert_with(self.name, Default::default)
            .pin()
            .try_insert(
                core::mem::take(&mut self.generic_params),
                core::mem::take(&mut self.body),
            )
            .unwrap_or_else(|_| panic!("function {} is already defined", self.name));
    }
}
