use crate::*;
use std::collections::{HashMap, HashSet};

/// Map from symbol to it's possible sets of generic args.
type InstanceMap = HashMap<Symbol, HashSet<Vec<Type>>>;

/// Monomorphization context.
struct Context<'a> {
    types: InstanceMap,
    functions: InstanceMap,
    /// Guard on [`Module::types`]
    type_guard: papaya::LocalGuard<'a>,
    func_guard: papaya::LocalGuard<'a>,
}

fn exists(instances: &mut InstanceMap, name: Symbol, args: &[Type]) -> bool {
    let instances = instances.entry(name).or_default();
    if instances.contains(args) {
        return true;
    }

    instances.insert(args.to_vec());
    false
}

/// Compute instances from a type decl.
fn visit_ty(module: &Module, ctx: &mut Context, ty: &mut Type) {
    match ty {
        Type::Symbol(name, args) if !args.is_empty() => {
            let moname = module.monomorphized_name(*name, args);
            if !exists(&mut ctx.types, *name, args) {
                let mut new_ty = module.get_ty(*name, &ctx.type_guard).instantiate(&args);
                visit_ty(module, ctx, &mut new_ty);
                module.types.pin().insert(
                    moname,
                    TypeAlias {
                        generics: Vec::new(),
                        type_: new_ty,
                    }
                    .into(),
                );
            }

            *ty = Type::Symbol(moname, Vec::new());
        }
        Type::Array(ty, _) => visit_ty(module, ctx, ty),
        Type::Struct { fields } => {
            for (_, ty) in fields {
                visit_ty(module, ctx, ty);
            }
        }
        Type::Ptr(ty, _) => visit_ty(module, ctx, ty),
        Type::FnPtr {
            params,
            return_type,
        } => {
            for ty in params {
                visit_ty(module, ctx, ty);
            }

            if let Some(ty) = return_type {
                visit_ty(module, ctx, ty);
            }
        }
        Type::Param(param) => {
            panic!("[bug] generic param #{param} encountered while computing used generic symbols")
        }
        _ => (),
    }
}

/// Compute type instances from a function.
fn visit_function(module: &Module, ctx: &mut Context, func: &mut Function) {
    for (_, ty) in &mut func.params {
        visit_ty(module, ctx, ty);
    }

    if let Some(ty) = &mut func.return_type {
        visit_ty(module, ctx, ty);
    }

    if let Some(body) = &mut func.body {
        for var in &mut body.variables {
            visit_ty(module, ctx, &mut var.ty);
        }

        for symbol in &mut body.symbols {
            if symbol.generics.is_empty() {
                continue;
            }

            let moname = module.monomorphized_name(symbol.name, &symbol.generics);
            if !exists(&mut ctx.functions, symbol.name, &symbol.generics) {
                let mut func = module.get_symbol(symbol.name, &ctx.func_guard).clone();
                func.instantiate(&symbol.generics);
                visit_function(module, ctx, &mut func);
                module
                    .functions
                    .insert(moname, func.into(), &ctx.func_guard);
            }

            symbol.name = moname;
            symbol.generics.clear();
        }
    }
}

impl Module {
    /// Get a name for a monomorphized version of a symbol.
    pub fn monomorphized_name(&self, name: Symbol, args: &[Type]) -> Symbol {
        if args.is_empty() {
            name
        } else {
            format!("{name}{}", crate::types::fmt_generic_args(args)).into()
        }
    }

    /// Monomorphize the module (duplicate generic symbols for all usages).
    pub fn monomorphize(&mut self) {
        let mut ctx = Context {
            types: HashMap::new(),
            functions: HashMap::new(),
            type_guard: self.types.guard(),
            func_guard: self.functions.guard(),
        };

        let types = self.types.pin();
        for alias in types.values() {
            let mut alias = alias.write().unwrap();
            if !alias.generics.is_empty() {
                continue;
            }

            visit_ty(self, &mut ctx, &mut alias.type_);
        }

        let functions = self.functions.pin();
        for func in functions.values() {
            let mut func = func.write().unwrap();
            if !func.generics.is_empty() {
                continue;
            }

            visit_function(self, &mut ctx, &mut func);
        }

        for (name, _) in ctx.types {
            types.remove(&name);
        }

        for (name, _) in ctx.functions {
            functions.remove(&name);
        }
    }
}
