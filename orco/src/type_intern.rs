//! Type interning middleware, see [`TypeIntern`]
//! Also the main example for making a middleware

use crate::{Backend, BodyCodegen, Symbol, Type};

/// Replaces all occurences of anonymous structs by named structs
pub struct TypeIntern<B: Backend> {
    backend: B,
    // TODO: Async container
    interned: scc::HashSet<Symbol>,
}

impl<B: Backend> std::ops::Deref for TypeIntern<B> {
    type Target = B;

    fn deref(&self) -> &Self::Target {
        &self.backend
    }
}

impl<B: Backend> std::ops::DerefMut for TypeIntern<B> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.backend
    }
}

impl<B: Backend> TypeIntern<B> {
    /// Wrap an existing backend
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            interned: scc::HashSet::new(),
        }
    }

    fn intern_type(&self, ty: &mut Type, root: bool) {
        match ty {
            Type::Array(ty, _) => self.intern_type(ty.as_mut(), true),
            Type::Struct(fields) if !root => {
                for (_, ty) in fields {
                    self.intern_type(ty, true);
                }
            }
            Type::Struct(..) if root => {
                let sym = Symbol::new(&format!("s {}", ty.hashable_name()));
                let ty = std::mem::replace(ty, Type::Symbol(sym));
                if self.interned.insert_sync(sym).is_ok() {
                    self.backend.type_(sym, ty);
                }
            }
            _ => (),
        }
    }
}

impl<B: Backend> crate::PrimitiveTypeSource for TypeIntern<B> {
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

impl<B: Backend> Backend for TypeIntern<B> {
    fn function(
        &self,
        name: Symbol,
        mut params: Vec<(Option<Symbol>, Type)>,
        mut return_type: Type,
    ) -> impl crate::codegen::BodyCodegen<'_> {
        for (_, ty) in &mut params {
            self.intern_type(ty, true);
        }
        self.intern_type(&mut return_type, true);
        let codegen = self.backend.function(name, params, return_type);
        Codegen(self, codegen)
    }

    fn type_(&self, name: Symbol, mut ty: Type) {
        self.intern_type(&mut ty, false);
        self.backend.type_(name, ty);
    }
}

// Codegen
struct Codegen<'a, B: Backend, CG: BodyCodegen<'a>>(&'a TypeIntern<B>, CG);

impl<'a, B: Backend, CG: BodyCodegen<'a>> std::ops::Deref for Codegen<'a, B, CG> {
    type Target = CG;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl<'a, B: Backend, CG: BodyCodegen<'a>> std::ops::DerefMut for Codegen<'a, B, CG> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.1
    }
}

impl<'a, B: Backend, CG: BodyCodegen<'a>> BodyCodegen<'a> for Codegen<'a, B, CG> {
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
        self.0.intern_type(&mut ty, true);
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
