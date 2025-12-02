//! Type interning middleware, see [TypeIntern]
//! Also the main example for making a middleware
use crate::{Symbol, Type};

/// Replaces all occurences of anonymous structs by named structs
#[derive(Debug, Default)]
pub struct TypeInterner {
    interned: scc::HashSet<Symbol>,
}

impl TypeInterner {
    #[allow(missing_docs)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Call this method on every type you want interning to happen (including typedefs)
    pub fn on_type(&self, backend: &impl crate::Backend, ty: &mut Type, named: bool) {
        match ty {
            Type::Array(ty, _) => self.on_type(backend, ty.as_mut(), true),
            Type::Struct(fields) if named => {
                for (_, ty) in fields {
                    self.on_type(backend, ty, true);
                }
            }
            Type::Struct(..) => {
                let sym = Symbol::new(&format!("s {}", ty.hashable_name()));
                let ty = std::mem::replace(ty, Type::Symbol(sym));
                if self.interned.insert_sync(sym).is_ok() {
                    backend.type_(sym, ty);
                }
            }
            _ => (),
        }
    }

    pub fn unit_sym(&self) -> Symbol {
        "s ".into()
    }

    /// Generate a unit type (interned)
    pub fn unit_ty(&self) -> Type {
        Type::Symbol(self.unit_sym())
    }

    /// Check if this type (has been interned before) is a unit type (aka an empty struct)
    pub fn is_unit(&self, ty: &Type) -> bool {
        ty == &self.unit_ty()
    }
}
