use crate::{Module, Type, ty::AliasId};
use std::collections::HashMap;

/// Replaces all anonymous structs by named structs, considers `ty`
/// named if `root` is true.
fn name_anonymous(
    module: &mut Module,
    map: &mut HashMap<Type, AliasId>,
    generics: &[crate::TyParam],
    ty: &mut Type,
    root: bool,
) {
    match ty {
        Type::Alias(_, alias_generics) => {
            for ty in alias_generics {
                name_anonymous(module, map, generics, ty, false);
            }
        }
        Type::Array(ty, _) => name_anonymous(module, map, generics, ty, false),
        Type::Struct { fields } => {
            for (_, ty) in fields {
                name_anonymous(module, map, generics, ty, false);
            }
        }
        Type::Ptr(ty, _) => name_anonymous(module, map, generics, ty, false),
        Type::FnPtr { types } => {
            for ty in types {
                name_anonymous(module, map, generics, ty, false);
            }
        }
        _ => (),
    }

    if let Some(id) = map.get(ty) {
        *ty = Type::Alias(*id, (0..generics.len() as u32).map(Type::Param).collect());
        return;
    }

    // Name anonymous
    let name = match ty {
        _ if root => return,
        Type::Struct { fields } => fields
            .iter()
            .map(|(_, ty)| module.display(ty).to_string())
            .collect(),
        _ => return,
    };

    // Put it in
    let id = super::create_id(&module.types);
    let ty = std::mem::replace(
        ty,
        Type::Alias(id, (0..generics.len() as u32).map(Type::Param).collect()),
    );

    map.insert(ty.clone(), id);
    module.types.insert(
        id,
        crate::ty::Alias {
            name: Some(name),
            generics: generics.to_vec(),
            ty,
        },
    );
}

/// Name all anonymous types in a symbol.
fn visit_symbol(module: &mut Module, map: &mut HashMap<Type, AliasId>, symbol: &mut crate::Symbol) {
    let crate::Symbol::Function(func) = symbol;
    for (_, ty) in &mut func.params {
        name_anonymous(module, map, &func.generics, ty, false);
    }

    name_anonymous(module, map, &func.generics, &mut func.return_type, false);

    let Some(body) = &mut func.body else {
        return;
    };

    for var in &mut body.variables {
        name_anonymous(module, map, &func.generics, &mut var.ty, false);
    }

    for instr in &mut body.instructions {
        use crate::ir::Instr;
        match instr {
            Instr::Cast(ty) | Instr::Transmute(ty) => {
                name_anonymous(module, map, &func.generics, ty, false)
            }
            _ => (),
        }
    }
}

impl Module {
    /// Replaces all anonymous structs by named structs.
    pub fn name_anonymous_structs(&mut self) {
        let mut map = HashMap::new();

        let types = self.types.keys().copied().collect::<Vec<_>>();
        for id in types {
            let mut alias = self.types.remove(&id).unwrap();
            name_anonymous(self, &mut map, &alias.generics, &mut alias.ty, true);
            self.types.insert(id, alias);
        }

        let symbols = self.symbols.keys().copied().collect::<Vec<_>>();
        for id in symbols {
            let mut symbol = self.symbols.remove(&id).unwrap();
            visit_symbol(self, &mut map, &mut symbol);
            self.symbols.insert(id, symbol);
        }
    }
}
