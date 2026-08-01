//! Intermediate representation backend for orco. Does
//! not compile to anything, just a way to store the code.
//! See [Store]
#![warn(missing_docs)]

/// Intermediate representation for code
pub mod ir;

// /// Code generation impl
// pub mod codegen;

/// Utilities to work with generics and specializations
pub mod generics;

// /// IR forwarding - invoking another backend
// /// to generate code from the IR
// mod forwarding;

use generics::Specialized;
use papaya::{HashMap, HashSet};

/// Function declaration, see [`Store::functions`]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Function {
    #[allow(missing_docs)]
    pub generic_params: Vec<orco::Type>,
    #[allow(missing_docs)]
    pub signature: orco::types::FunctionSignature,
    /// List of definitions (generic specializations)
    pub bodies: Specialized<ir::Body>,
}

/// The heart storage
#[derive(Clone, Debug, Default)]
pub struct Store {
    /// Type aliases
    pub types: HashMap<orco::Symbol, Specialized<orco::Type>>,
    /// Function declarations
    pub functions: HashMap<orco::Symbol, Function>,

    /// List of generic params to monomorphize types
    type_instances: HashSet<(orco::Symbol, Vec<orco::Type>)>,
    /// List of generic params to monomorphize functions
    function_instances: HashSet<(orco::Symbol, Vec<orco::Type>)>,
}

impl Store {
    #[allow(missing_docs)]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// If `ty` is a type alias, will be replaced by what is aliased.
    /// Inner aliases (f.e. struct field types) are not replaced!
    pub fn inline_type_aliases(&self, mut ty: orco::Type) -> orco::Type {
        let types = self.types.pin();
        while let orco::Type::Symbol(name, generics) = ty {
            let specs = types
                .get(&name)
                .unwrap_or_else(|| panic!("undeclared type {name}"));
            ty = generics::match_specialization(specs, &generics, self, |ty, map| {
                let mut ty = ty.clone();
                ty.instantiate(&map);
                ty
            })
            .unwrap_or_else(move || {
                panic!(
                    "no matching specialization for type {}",
                    orco::Type::Symbol(name, generics)
                )
            });
        }
        ty
    }

    /// Find a best-matching type for a set of generics
    pub fn get_type(
        &self,
        name: orco::Symbol,
        generics: &[orco::Type],
        callback: impl FnOnce(&orco::Type, generics::TypeMap),
    ) {
        let types = self.types.pin();
        let specs = types
            .get(&name)
            .unwrap_or_else(|| panic!("undeclared type {name}"));
        generics::match_specialization(&specs, generics, self, callback).unwrap_or_else(|| {
            panic!(
                "no matching specialization for {name}{}",
                orco::types::fmt_generic_args(generics)
            )
        })
    }

    /// Find a best-matching function body for a set of generics
    pub fn get_function_body(
        &self,
        name: orco::Symbol,
        generics: &[orco::Type],
        callback: impl FnOnce(&ir::Body, generics::TypeMap),
    ) {
        let bodies = self.function_bodies.pin();
        let specs = bodies
            .get(&name)
            .unwrap_or_else(|| panic!("undeclared function {name}"));
        generics::match_specialization(&specs, generics, self, callback).unwrap_or_else(|| {
            panic!(
                "no matching specialization for {name}{}",
                orco::types::fmt_generic_args(generics)
            )
        })
    }
}

impl orco::DeclarationBackend for Store {
    fn function(
        &self,
        name: orco::Symbol,
        generic_params: Vec<orco::Type>,
        params: Vec<(Option<String>, orco::Type)>,
        return_type: Option<orco::Type>,
        attrs: orco::attrs::FunctionAttributes,
    ) {
        self.functions
            .pin()
            .try_insert(
                name,
                FunctionDecl {
                    generic_params,
                    signature: orco::types::FunctionSignature {
                        params,
                        return_type,
                        attrs,
                    },
                },
            )
            .unwrap_or_else(|_| panic!("function {name} is already declared"));
    }

    fn type_(&self, name: orco::Symbol, generic_params: Vec<orco::Type>, ty: orco::Type) {
        self.types
            .pin()
            .get_or_insert_with(name, Default::default)
            .pin()
            .try_insert(generic_params, ty)
            .unwrap_or_else(|_| panic!("type {name} is already declared"));
    }
}

impl std::fmt::Display for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (name, specs) in self.types.pin().iter() {
            for (spec, ty) in specs.pin().iter() {
                writeln!(
                    f,
                    "type {name}{} = {ty};",
                    orco::types::fmt_generic_args(spec)
                )?;
            }
        }

        writeln!(f)?;

        let bodies = self.function_bodies.pin();
        for (name, decl) in self.functions.pin().iter() {
            writeln!(
                f,
                "{}fn {name}{}{};",
                decl.signature.attrs,
                orco::types::fmt_generic_args(&decl.generic_params),
                decl.signature,
            )?;

            let Some(defs) = bodies.get(name) else {
                continue;
            };

            for (spec, body) in defs.pin().iter() {
                writeln!(f, "for {} {body}", orco::types::fmt_generic_args(spec)).unwrap();
            }

            writeln!(f)?;
        }

        Ok(())
    }
}
