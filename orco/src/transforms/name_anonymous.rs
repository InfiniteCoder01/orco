use crate::*;
use std::collections::HashSet;

/// Replaces all anonymous structs by named structs, considers `ty`
/// named if `root` is true.
fn name_anonymous(
    types: &SymbolMapRef<TypeAlias>,
    generics: &mut HashSet<Symbol>,
    ty: &mut Type,
    root: bool,
) {
    match ty {
        Type::Symbol(_, symbol_generics) => {
            for ty in symbol_generics {
                name_anonymous(types, generics, ty, false);
            }
        }
        Type::Array(ty, _) => name_anonymous(types, generics, ty, false),
        Type::Struct { fields } => {
            for (_, ty) in fields {
                name_anonymous(types, generics, ty, false);
            }
        }
        Type::Ptr(ty, _) => name_anonymous(types, generics, ty, false),
        Type::FnPtr {
            params,
            return_type,
        } => {
            for ty in params {
                name_anonymous(types, generics, ty, false);
            }

            if let Some(ty) = return_type {
                name_anonymous(types, generics, ty, false);
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
    types.insert(
        name,
        TypeAlias {
            generics,
            type_: ty,
        },
    );
}

impl Module {
    /// Replaces all anonymous structs by named structs.
    pub fn name_anonymous_structs(&self) {
        let types = self.types.pin();
        for (name, alias) in types.iter() {
            let mut alias = alias.clone();
            name_anonymous(&types, &mut HashSet::new(), &mut alias.type_, true);
            types.insert(*name, alias);
        }

        let functions = self.functions.pin();
        for (name, func) in functions.iter() {
            let mut func = func.clone();
            for (_, ty) in &mut func.params {
                name_anonymous(&types, &mut HashSet::new(), ty, false);
            }

            if let Some(ty) = &mut func.return_type {
                name_anonymous(&types, &mut HashSet::new(), ty, false);
            }

            functions.insert(*name, func);
        }
    }
}
