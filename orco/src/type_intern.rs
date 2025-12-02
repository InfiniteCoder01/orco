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
}

impl TypeInterner {
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
}
