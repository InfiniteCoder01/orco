use crate::*;
use std::collections::HashSet;

/// Replaces all anonymous structs by named structs, considers `ty`
/// named if `root` is true.
fn name_anonymous(
    module: &Module,
    guard: &impl papaya::Guard,
    generics: &mut HashSet<Symbol>,
    ty: &mut Type,
    root: bool,
) {
    match ty {
        Type::Symbol(_, symbol_generics) => {
            for ty in symbol_generics {
                name_anonymous(module, guard, generics, ty, false);
            }
        }
        Type::Array(ty, _) => name_anonymous(module, guard, generics, ty, false),
        Type::Struct { fields } => {
            for (_, ty) in fields {
                name_anonymous(module, guard, generics, ty, false);
            }
        }
        Type::Ptr(ty, _) => name_anonymous(module, guard, generics, ty, false),
        Type::FnPtr {
            params,
            return_type,
        } => {
            for ty in params {
                name_anonymous(module, guard, generics, ty, false);
            }

            if let Some(ty) = return_type {
                name_anonymous(module, guard, generics, ty, false);
            }
        }
        Type::Param(param) => {
            generics.insert(*param);
        }
        _ => (),
    }

    if root {
        return;
    }

    let name = match ty {
        Type::Struct { fields } => fields
            .iter()
            .map(|(_, ty)| ty.to_string())
            .collect::<String>()
            .into(),
        _ => return,
    };

    let generics = generics.iter().copied().collect::<Vec<_>>();
    let ty = std::mem::replace(
        ty,
        Type::Symbol(name, generics.iter().copied().map(Type::Param).collect()),
    );

    module.types.insert(
        name,
        TypeAlias {
            generics,
            type_: ty,
        }
        .into(),
        guard,
    );
}

impl Module {
    /// Replaces all anonymous structs by named structs.
    pub fn name_anonymous_structs(&mut self) {
        let guard = self.types.guard();
        for (_, alias) in self.types.iter(&guard) {
            let mut alias = alias.write().unwrap();
            name_anonymous(self, &guard, &mut HashSet::new(), &mut alias.type_, true);
        }

        for (_, func) in self.functions.pin().iter() {
            let mut func = func.write().unwrap();
            for (_, ty) in &mut func.params {
                name_anonymous(self, &guard, &mut HashSet::new(), ty, false);
            }

            if let Some(ty) = &mut func.return_type {
                name_anonymous(self, &guard, &mut HashSet::new(), ty, false);
            }

            if let Some(body) = &mut func.body {
                for var in &mut body.variables {
                    name_anonymous(self, &guard, &mut HashSet::new(), &mut var.ty, false);
                }
            }
        }
    }
}
