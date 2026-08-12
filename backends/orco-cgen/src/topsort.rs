use std::collections::HashMap;

/// Adds all symbols this type uses into `dependencies`
fn type_dependencies(ty: &orco::Type, dependencies: &mut Vec<orco::Symbol>) {
    match ty {
        orco::Type::Symbol(name, generics) => {
            dependencies.push(*name) // TODO: generics
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

type MapRef<'a> = orco::papaya::HashMapRef<
    'a,
    orco::Symbol,
    orco::TypeAlias,
    std::hash::RandomState,
    orco::papaya::LocalGuard<'a>,
>;

/// `visited` is a map, where if name isn't present - not visited,
/// otherwise stores whether it has finished processing
/// (if false is encountered, loop is detected)
fn topsort<E>(
    visited: &mut HashMap<orco::Symbol, bool>,
    types: &MapRef,
    callback: &mut impl FnMut(orco::Symbol, &orco::Type) -> Result<(), E>,
    name: orco::Symbol,
) -> Result<(), E> {
    use std::collections::hash_map::Entry;
    match visited.entry(name) {
        Entry::Occupied(finished) if *finished.get() => return Ok(()),
        Entry::Occupied(_) => panic!(
            "type dependency cycle detected on {name}, possibly an infinitely-recursive type",
        ),
        Entry::Vacant(entry) => entry.insert(false),
    };

    let ty = &types
        .get(&name)
        .unwrap_or_else(|| panic!("[bug] undeclared type {name}"))
        .type_;
    let mut dependencies = Vec::new();
    type_dependencies(ty, &mut dependencies);

    for dep in dependencies {
        topsort(visited, types, callback, dep);
    }

    callback(name, ty)?;
    visited.insert(name, true);
    Ok(())
}

/// Visit types in topological order (excluding pointers).
pub fn visit<E>(
    module: &orco::Module,
    mut callback: impl FnMut(orco::Symbol, &orco::Type) -> Result<(), E>,
) -> Result<(), E> {
    let types = module.types.pin();
    let mut visited = HashMap::new();

    for (name, _) in types.iter() {
        topsort(&mut visited, &types, &mut callback, *name)?;
    }

    Ok(())
}
