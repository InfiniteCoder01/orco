use crate::*;
use std::collections::{HashMap, HashSet};

/// Map from symbol to it's possible sets of generic args.
type InstanceMap = HashMap<Symbol, HashSet<Vec<Type>>>;

/// Monomorphization context.
struct Context {
    types: InstanceMap,
    functions: InstanceMap,
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
                let mut new_ty = module
                    .types
                    .pin()
                    .get(name)
                    .unwrap_or_else(|| panic!("undelcared type {name}"))
                    .instantiate(&args);
                visit_ty(module, ctx, &mut new_ty);
                module.types.pin().insert(
                    moname,
                    TypeAlias {
                        generics: Vec::new(),
                        type_: new_ty,
                    },
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
fn visit_function(module: &Module, ctx: &mut Context, name: Symbol, args: &[Type]) {
    let functions = module.functions.pin();
    let func = functions
        .get(&name)
        .unwrap_or_else(|| panic!("undeclared function {name}"));

    let mut type_params = func.type_params.clone();
    type_params.extend(func.generics.iter().copied().zip(args.iter().cloned()));

    let params = func
        .params
        .iter()
        .map(|(name, ty)| {
            let mut ty = ty.copy_instantiate(&type_params);
            visit_ty(module, ctx, &mut ty);
            (name.clone(), ty)
        })
        .collect::<Vec<_>>();
    let return_type = func.return_type.as_ref().map(|ty| {
        let mut ty = ty.copy_instantiate(&type_params);
        visit_ty(module, ctx, &mut ty);
        ty
    });

    if let Some(body) = func.body.get() {
        for var in &body.variables {
            visit_ty(module, ctx, &mut var.ty.clone());
        }

        for symbol in &body.symbols {
            visit_function(module, ctx, symbol.name, &symbol.generics);
        }
    }

    functions.insert(
        module.monomorphized_name(name, args),
        Function {
            generics: Vec::new(),
            type_params,
            params,
            return_type,
            attrs: func.attrs.clone(),
            body: func.body.clone(),
        },
    );
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
    pub fn monomorphize(&self) {
        let mut ctx = Context {
            types: HashMap::new(),
            functions: HashMap::new(),
        };

        let types = self.types.pin();
        for (name, alias) in types.iter() {
            if !alias.generics.is_empty() {
                continue;
            }

            let mut alias = alias.clone();
            visit_ty(self, &mut ctx, &mut alias.type_);
            types.insert(*name, alias);
        }

        let functions = self.functions.pin();
        for (name, func) in functions.iter() {
            if !func.generics.is_empty() {
                continue;
            }

            visit_function(self, &mut ctx, *name, &[]);
        }

        for (name, _) in ctx.types {
            types.remove(&name);
        }

        for (name, _) in ctx.functions {
            functions.remove(&name);
        }
    }
}
