use orco::codegen as oc;

/// Function
#[derive(Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct Function {}

/// Implementation of [`orco::BodyCodegen`]
pub struct Codegen<'a, 'b: 'a> {
    /// Backend context that will recieve the symbol once codegen is done
    pub backend: &'a crate::Backend<'b>,
    /// Symbol name
    pub name: orco::Symbol,
}

impl<'a, 'b: 'a> Codegen<'a, 'b> {
    pub fn new(backend: &'a crate::Backend<'b>, name: orco::Symbol) -> Self {
        Self { backend, name }
    }
}

impl oc::BodyCodegen for Codegen<'_, '_> {
    fn type_of(&self, id: usize) -> orco::Type {
        todo!()
    }

    fn declare_var(&mut self, ty: orco::Type) -> oc::Variable {
        todo!()
    }

    fn arg_var(&self, idx: usize) -> oc::Variable {
        todo!()
    }

    fn assign(&mut self, target: oc::Place, value: oc::Value) {
        todo!()
    }

    fn iconst(&mut self, value: i128, size: orco::types::IntegerSize) -> oc::Value {
        todo!()
    }

    fn uconst(&mut self, value: u128, size: orco::types::IntegerSize) -> oc::Value {
        todo!()
    }

    fn fconst(&mut self, value: f64, size: u16) -> oc::Value {
        todo!()
    }

    fn bconst(&mut self, value: bool) -> oc::Value {
        todo!()
    }

    fn read(&mut self, place: oc::Place) -> oc::Value {
        todo!()
    }

    fn reference(&mut self, place: oc::Place) -> oc::Value {
        todo!()
    }

    fn call(&mut self, func: oc::Value, args: Vec<oc::Value>) -> Option<oc::Value> {
        todo!()
    }

    fn return_(&mut self, value: Option<oc::Value>) {
        todo!()
    }
}
