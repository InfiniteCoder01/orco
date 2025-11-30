//! Type interning middleware, see [TypeIntern]
//! Also the main example for making a middleware
use crate::{Symbol, Type};

/// Replaces all occurences of anonymous structs by named structs
pub struct TypeIntern {
    interned: scc::HashSet<Symbol>,
}

impl TypeIntern {
    pub fn new() -> Self {
        Self {
            interned: scc::HashSet::new(),
        }
    }
}

impl crate::Middleware for TypeIntern {
    fn on_type(&self, backend: &impl crate::Backend, ty: &mut Type, anonymous: bool) {
        match ty {
            Type::Array(ty, _) => self.on_type(backend, ty.as_mut(), true),
            Type::Struct(fields) if !anonymous => {
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
}
