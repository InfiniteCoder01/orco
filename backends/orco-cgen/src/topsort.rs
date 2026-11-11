use orco::ty::AliasId;
use std::collections::HashMap;

/// Adds all symbols this type uses into `dependencies`
fn type_dependencies(ty: &orco::Type, dependencies: &mut Vec<AliasId>) {
    match ty {
        orco::Type::Alias(id, generics) => {
            assert!(
                generics.is_empty(),
                "generics type encountered in C backend ({ty:?}), did you forget to monomorphize types?",
            );
            dependencies.push(*id);
        }
        orco::Type::Array(ty, sz) if *sz > 0 => type_dependencies(ty, dependencies),
        orco::Type::Struct { fields } => {
            for (_, ty) in fields {
                type_dependencies(ty, dependencies);
            }
        }
        _ => (),
    }
}

/// `visited` is a map, where if name isn't present - not visited,
/// otherwise stores whether it has finished processing
/// (if false is encountered, loop is detected)
fn topsort<E>(
    module: &orco::Module,
    visited: &mut HashMap<AliasId, bool>,
    callback: &mut impl FnMut(AliasId, &orco::ty::Alias) -> Result<(), E>,
    id: AliasId,
) -> Result<(), E> {
    use std::collections::hash_map::Entry;
    match visited.entry(id) {
        Entry::Occupied(finished) if *finished.get() => return Ok(()),
        Entry::Occupied(_) => panic!(
            "type dependency cycle detected on {}, possibly an infinitely-recursive type",
            module.display(&id)
        ),
        Entry::Vacant(entry) => entry.insert(false),
    };

    let Some(ty) = module.types.get(&id) else {
        panic!("invalid {id:?}");
    };

    let mut dependencies = Vec::new();
    type_dependencies(&ty.ty, &mut dependencies);

    for dep in dependencies {
        topsort(module, visited, callback, dep)?;
    }

    callback(id, ty)?;
    visited.insert(id, true);
    Ok(())
}

/// Visit types in topological order (excluding pointers).
pub fn visit<E>(
    module: &orco::Module,
    mut callback: impl FnMut(AliasId, &orco::ty::Alias) -> Result<(), E>,
) -> Result<(), E> {
    let mut visited = HashMap::new();
    for id in module.types.keys() {
        topsort(&module, &mut visited, &mut callback, *id)?;
    }

    Ok(())
}
