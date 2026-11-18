use crate::{Module, Type};

/// Replaces all anonymous structs by named structs, considers `ty`
/// named if `root` is true.
fn name_anonymous(module: &mut Module, generics: &[crate::TyParam], ty: &mut Type, root: bool) {
    match ty {
        Type::Alias(_, alias_generics) => {
            for ty in alias_generics {
                name_anonymous(module, generics, ty, false);
            }
        }
        Type::Array(ty, _) => name_anonymous(module, generics, ty, false),
        Type::Struct { fields } => {
            for (_, ty) in fields {
                name_anonymous(module, generics, ty, false);
            }
        }
        Type::Ptr(ty, _) => name_anonymous(module, generics, ty, false),
        Type::FnPtr { types } => {
            for ty in types {
                name_anonymous(module, generics, ty, false);
            }
        }
        _ => (),
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
fn visit_symbol(module: &mut Module, symbol: &mut crate::Symbol) {
    let crate::Symbol::Function(func) = symbol;
    for (_, ty) in &mut func.params {
        name_anonymous(module, &func.generics, ty, false);
    }

    name_anonymous(module, &func.generics, &mut func.return_type, false);

    let Some(body) = &mut func.body else {
        return;
    };

    for var in &mut body.variables {
        name_anonymous(module, &func.generics, &mut var.ty, false);
    }

    for instr in &mut body.instructions {
        use crate::ir::Instr;
        match instr {
            Instr::Cast(ty) | Instr::Transmute(ty) => {
                name_anonymous(module, &func.generics, ty, false)
            }
            _ => (),
        }
    }
}

impl Module {
    /// Replaces all anonymous structs by named structs.
    pub fn name_anonymous_structs(&mut self) {
        let types = self.types.keys().copied().collect::<Vec<_>>();
        for id in types {
            let mut alias = self.types.remove(&id).unwrap();
            name_anonymous(self, &alias.generics, &mut alias.ty, true);
            self.types.insert(id, alias);
        }

        let symbols = self.symbols.keys().copied().collect::<Vec<_>>();
        for id in symbols {
            let mut symbol = self.symbols.remove(&id).unwrap();
            visit_symbol(self, &mut symbol);
            self.symbols.insert(id, symbol);
        }
    }
}
