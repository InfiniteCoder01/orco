use crate::{Module, SymbolId, Type, ty::AliasId};
use std::collections::HashMap;

/// Monomorphization context.
struct Context {
    types: HashMap<AliasId, HashMap<Vec<Type>, AliasId>>,
    symbols: HashMap<SymbolId, HashMap<Vec<Type>, SymbolId>>,
}

fn mono_id<K, V>(
    store: &HashMap<K, V>,
    instances: &mut HashMap<K, HashMap<Vec<Type>, K>>,
    id: K,
    args: &[Type],
) -> (K, bool)
where
    K: Copy + std::hash::Hash + Eq + From<usize>,
{
    let instances = instances.entry(id).or_default();
    if let Some(id) = instances.get(args) {
        return (*id, false);
    }

    // Uh-oh...
    let mut id = 0;
    for key in store.keys().take(100) {
        use std::hash::BuildHasher;
        id ^= store.hasher().hash_one(key);
    }

    loop {
        let key = K::from(id as _);
        if !store.contains_key(&key) {
            instances.insert(args.to_vec(), key);
            return (key, true);
        }

        id += 1;
    }
}

fn mono_name(module: &Module, name: Option<&str>, args: &[Type]) -> Option<String> {
    name.map(|name| {
        use std::fmt::Write as _;
        let mut name = name.to_owned();
        for arg in args {
            write!(&mut name, "#{}", module.display(arg)).unwrap();
        }

        name
    })
}

/// Compute instances from a type decl.
fn visit_ty(module: &mut Module, ctx: &mut Context, ty: &mut Type) {
    match ty {
        Type::Alias(id, generics) if !generics.is_empty() => {
            let (monoid, new) = mono_id(&module.types, &mut ctx.types, *id, generics);
            if new {
                let alias = &module.types[id];
                let moname = mono_name(module, alias.name.as_deref(), generics);
                let mut moty = alias.ty.copy_instantiate(&generics);
                visit_ty(module, ctx, &mut moty);
                module.types.insert(
                    monoid,
                    crate::ty::Alias {
                        name: moname,
                        generics: Vec::new(),
                        ty: moty,
                    }
                    .into(),
                );
            }

            *ty = Type::Alias(monoid, thin_vec::ThinVec::new());
        }
        Type::Array(ty, _) => visit_ty(module, ctx, ty),
        Type::Struct { fields } => {
            for (_, ty) in fields {
                visit_ty(module, ctx, ty);
            }
        }
        Type::Ptr(ty, _) => visit_ty(module, ctx, ty),
        Type::FnPtr { types } => {
            for ty in types {
                visit_ty(module, ctx, ty);
            }
        }
        Type::Param(param) => {
            panic!("[bug] generic param #{param} encountered while computing used generic symbols")
        }
        _ => (),
    }
}

fn instantiate_symbol(
    module: &mut Module,
    ctx: &mut Context,
    id: SymbolId,
    generics: &[Type],
) -> SymbolId {
    let (monoid, new) = mono_id(&module.symbols, &mut ctx.symbols, id, generics);
    if new {
        let mut symbol = module.symbols[&id].clone();
        let moname = mono_name(module, symbol.name(), generics);

        let crate::Symbol::Function(func) = &mut symbol;
        func.name = moname;
        func.instantiate(generics);

        visit_symbol(module, ctx, &mut symbol);
        module.symbols.insert(monoid, symbol);
    }

    monoid
}

/// Compute type instances from a symbol.
fn visit_symbol(module: &mut Module, ctx: &mut Context, symbol: &mut crate::Symbol) {
    let crate::Symbol::Function(func) = symbol;
    for (_, ty) in &mut func.params {
        visit_ty(module, ctx, ty);
    }

    visit_ty(module, ctx, &mut func.return_type);

    let Some(body) = &mut func.body else {
        return;
    };

    for var in &mut body.variables {
        visit_ty(module, ctx, &mut var.ty);
    }

    for instr in &mut body.instructions {
        use crate::ir::Instr;
        match instr {
            Instr::Global(id, generics) if !generics.is_empty() => {
                *id = instantiate_symbol(module, ctx, *id, generics);
                generics.clear();
            }
            Instr::Cast(ty) | Instr::Transmute(ty) => visit_ty(module, ctx, ty),
            _ => (),
        }
    }
}

impl Module {
    /// Monomorphize the module (duplicate generic symbols for all usages).
    pub fn monomorphize(&mut self) {
        let mut ctx = Context {
            types: HashMap::new(),
            symbols: HashMap::new(),
        };

        let types = self.types.keys().copied().collect::<Vec<_>>();
        for id in types {
            let alias = &self.types[&id];
            if !alias.generics.is_empty() {
                continue;
            }

            let mut alias = self.types.remove(&id).unwrap();
            visit_ty(self, &mut ctx, &mut alias.ty);
            self.types.insert(id, alias);
        }

        let symbols = self.symbols.keys().copied().collect::<Vec<_>>();
        for id in symbols {
            let symbol = &self.symbols[&id];
            if !symbol.generics().is_empty() {
                continue;
            }

            let mut symbol = self.symbols.remove(&id).unwrap();
            visit_symbol(self, &mut ctx, &mut symbol);
            self.symbols.insert(id, symbol);
        }

        self.types.retain(|_, alias| alias.generics.is_empty());
        self.symbols
            .retain(|_, symbol| symbol.generics().is_empty());
    }
}
