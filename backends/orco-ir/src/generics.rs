// FIXME: Horrible
use crate::Store;
use orco::Type;

/// A map from a specialization (`Vec<Type>`, each type can hold named params) to the symbol
pub type Specialized<T> = papaya::HashMap<Vec<Type>, T>;

/// A type alias for a map from type parameter names to their types.
/// See [`match_ty`]
pub type TypeMap = std::collections::HashMap<orco::Symbol, Type>;

/// Match generic argument type to parameter type, inferring [`Type::Param`] and
/// writing it into `map`
pub fn match_ty(param: &Type, arg: &Type, map: &mut TypeMap, store: &Store) -> Option<()> {
    use Type::*;
    let param = store.inline_type_aliases(param.clone());
    let original_arg = arg.clone();
    let arg = store.inline_type_aliases(arg.clone());
    match (param, arg) {
        (param @ (Integer(_) | Unsigned(_) | Float(_) | Bool | Char(_)), arg) if arg == param => {
            Some(())
        }
        (Symbol(..), _) => unreachable!(),
        (Array(ty, size), Array(arg_ty, arg_size)) if arg_size == size => {
            match_ty(&ty, &arg_ty, map, store)
        }
        (Struct { fields }, Struct { fields: arg_fields }) if arg_fields.len() == fields.len() => {
            for ((name, ty), (arg_name, arg_ty)) in fields.iter().zip(arg_fields.iter()) {
                if name != arg_name {
                    return None;
                }
                match_ty(ty, arg_ty, map, store)?;
            }
            Some(())
        }
        (Ptr(ty, mutability), Ptr(arg_ty, arg_mutability)) if arg_mutability == mutability => {
            match_ty(&ty, &arg_ty, map, store)
        }
        (
            FnPtr {
                params,
                return_type,
            },
            FnPtr {
                params: arg_params,
                return_type: arg_return_type,
            },
        ) => todo!(),
        (Param(name), arg) if !matches!(original_arg, Error) => {
            map.insert(name, original_arg.clone());
            Some(())
        }
        _ => None,
    }
}

/// Matches a generic to argumens and returns the match map.
/// See [`match_ty`]
pub fn match_type_params(params: &[Type], args: &[Type], store: &Store) -> Option<TypeMap> {
    if params.len() != args.len() {
        return None;
    }

    let mut map = TypeMap::new();
    for (param, arg) in params.iter().zip(args.iter()) {
        match_ty(param, arg, &mut map, store)?
    }

    Some(map)
}

/// Find a specialization that matches best to set of generic arguments,
/// providing the matched type parameter map.
/// See also: [`match_ty`]
pub fn match_specialization<T, R>(
    specs: &Specialized<T>,
    args: &[Type],
    store: &Store,
    callback: impl FnOnce(&T, TypeMap) -> R,
) -> Option<R> {
    let specs = specs.pin();
    let mut best = None;
    for (params, spec) in specs.iter() {
        let Some(map) = match_type_params(params, args, store) else {
            continue;
        };

        if best
            .as_ref()
            .is_none_or(|(_, best_map): &(_, TypeMap)| map.len() > best_map.len())
        {
            best = Some((spec, map));
        }
    }

    best.map(|(spec, map)| callback(spec, map))
}

impl crate::FunctionDecl {
    /// See [Type::instantiate]
    pub fn instantiate(
        &self,
        store: &crate::Store,
        generic_args: &[Type],
    ) -> orco::types::FunctionSignature {
        let map = match_type_params(&self.generic_params, generic_args, store)
            .expect("failed to instantiate function decl: generics did not match");
        let mut sig = self.signature.clone();
        sig.instantiate(&map);
        sig
    }
}
