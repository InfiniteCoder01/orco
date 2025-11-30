//! Middleware API, see [Middleware]
use crate::{Backend, BodyCodegen, Symbol, Type};

/// Middleware trait, has entrypoints on different parts of compilation.
/// To use it, see [Intercept]
#[allow(unused_variables)]
pub trait Middleware: Sync {
    /// Called whenever a type is passed into the backend
    /// Type is considered anonymous unless it's passed into [`Backend::type_`]
    fn on_type(&self, backend: &impl Backend, ty: &mut Type, anonymous: bool) {}
}

impl<M1: Middleware, M2: Middleware> Middleware for (M1, M2) {
    fn on_type(&self, backend: &impl Backend, ty: &mut Type, anonymous: bool) {
        self.0.on_type(backend, ty, anonymous);
        self.1.on_type(backend, ty, anonymous);
    }
}

/// Wrapper around a backend that calls middleware functions
pub struct Intercept<B: Backend, M: Middleware> {
    backend: B,
    middleware: M,
}

impl<B: Backend, M: Middleware> std::ops::Deref for Intercept<B, M> {
    type Target = B;

    fn deref(&self) -> &Self::Target {
        &self.backend
    }
}

impl<B: Backend, M: Middleware> std::ops::DerefMut for Intercept<B, M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.backend
    }
}

impl<B: Backend, M: Middleware> Intercept<B, M> {
    /// Wrap an existing backend
    pub fn new(backend: B, middleware: M) -> Self {
        Self {
            backend,
            middleware,
        }
    }
}

impl<B: Backend, M: Middleware> crate::PrimitiveTypeSource for Intercept<B, M> {
    fn bool(&self) -> Type {
        self.backend.bool()
    }

    fn int(&self, size: u16, signedness: bool) -> Type {
        self.backend.int(size, signedness)
    }

    fn size_type(&self, signedness: bool) -> Type {
        self.backend.size_type(signedness)
    }

    fn float(&self, size: u16) -> Type {
        self.backend.float(size)
    }
}

impl<B: Backend, M: Middleware> Backend for Intercept<B, M> {
    fn function(
        &self,
        name: Symbol,
        mut params: Vec<(Option<Symbol>, Type)>,
        mut return_type: Type,
    ) -> impl crate::codegen::BodyCodegen<'_> {
        for (_, ty) in &mut params {
            self.middleware.on_type(&self.backend, ty, true);
        }
        self.middleware
            .on_type(&self.backend, &mut return_type, true);
        let codegen = self.backend.function(name, params, return_type);
        Codegen(self, codegen)
    }

    fn type_(&self, name: Symbol, mut ty: Type) {
        self.middleware.on_type(&self.backend, &mut ty, false);
        self.backend.type_(name, ty);
    }
}

// Codegen
struct Codegen<'a, B: Backend, M: Middleware, CG: BodyCodegen<'a>>(&'a Intercept<B, M>, CG);

impl<'a, B: Backend, M: Middleware, CG: BodyCodegen<'a>> BodyCodegen<'a> for Codegen<'a, B, M, CG> {
    fn external(self)
    where
        Self: Sized,
    {
        self.1.external()
    }

    fn comment(&mut self, comment: &str) {
        self.1.comment(comment)
    }

    fn declare_var(&mut self, mut ty: Type) -> crate::codegen::Variable {
        self.0.middleware.on_type(&self.0.backend, &mut ty, true);
        self.1.declare_var(ty)
    }

    fn arg_var(&self, idx: usize) -> crate::codegen::Variable {
        self.1.arg_var(idx)
    }

    fn assign(&mut self, value: crate::codegen::Operand, destination: crate::codegen::Place) {
        self.1.assign(value, destination)
    }

    fn call(
        &mut self,
        function: crate::codegen::Operand,
        args: Vec<crate::codegen::Operand>,
        destination: crate::codegen::Place,
    ) {
        self.1.call(function, args, destination)
    }

    fn return_(&mut self, value: crate::codegen::Operand) {
        self.1.return_(value)
    }
}
