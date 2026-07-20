//! C transpilation backend for orco.
//! Also used to generate C headers and
//! is generally the reference for other backends
//! See [Backend]
// TODO: ABI
// TODO: Extra type interning
#![warn(missing_docs)]

/// Type formatting & other things
pub mod types;
use types::FmtType;

/// Type interning and name conversion
mod type_names;

/// Symbol container types
pub mod symbols;

// /// Code generation, used to generate function bodies.
// pub mod codegen;
// pub use codegen::Codegen;

use papaya::HashMap;

/// Root backend struct
#[derive(Debug, Default)]
pub struct Backend {
    /// Type aliases
    pub types: HashMap<orco::Symbol, orco::Type>,
    /// Interned types
    interned: HashMap<orco::Type, orco::Symbol>,
    /// Function declarations
    pub functions: HashMap<orco::Symbol, orco::types::FunctionSignature>,
    /// Definitions
    definitions: std::sync::Mutex<Vec<String>>,
}

impl Backend {
    #[allow(missing_docs)]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a definition
    pub fn define(&self, code: String) {
        self.definitions.lock().unwrap().push(code);
    }

    /// Get the name of the symbol used in generated C code ("mangling")
    pub fn cname(&self, name: orco::Symbol) -> String {
        // Take only the method name, not the path
        // FIXME: conflicts...
        let mut new_name = String::new();
        for split in name.split([',', '<', '>', '{', '}']) {
            let split = &split[split.rfind([':', '.']).map_or(0, |i| i + 1)..];
            if !split.is_empty() {
                match new_name.chars().last() {
                    None | Some('_') => (),
                    _ => new_name.push('_'),
                }
                new_name.push_str(split);
            }
        }

        let mut new_name = new_name.replace(|c: char| !c.is_ascii_alphanumeric(), "_");
        if new_name.chars().next().is_none_or(|c| c.is_ascii_digit()) {
            new_name.insert(0, '_');
        }

        new_name
    }
}

impl orco::DeclarationBackend for Backend {
    fn function(
        &self,
        name: orco::Symbol,
        generics: Vec<orco::Type>,
        mut params: Vec<(Option<String>, orco::Type)>,
        mut return_type: Option<orco::Type>,
        attrs: orco::attrs::FunctionAttributes,
    ) {
        let name = self.generic_name(name, &generics);
        for (_, ty) in &mut params {
            self.intern_type(ty, None);
        }
        if let Some(rt) = &mut return_type {
            self.intern_type(rt, None);
        }
        self.functions
            .pin()
            .try_insert(
                name,
                orco::types::FunctionSignature {
                    params,
                    return_type,
                    attrs,
                },
            )
            .unwrap_or_else(|_| panic!("function {name} is already declared"));
    }

    fn type_(&self, name: orco::Symbol, generics: Vec<orco::Type>, mut ty: orco::Type) {
        let name = self.generic_name(name, &generics);
        self.intern_type(&mut ty, Some(name));
        self.types
            .pin()
            .try_insert(name, ty)
            .unwrap_or_else(|_| panic!("type {name} is already declared"));
    }
}

// impl orco::CodegenBackend for crate::Backend {
//     fn cg_function(
//         &self,
//         name: orco::Symbol,
//         generics: Vec<orco::Type>,
//     ) -> impl orco::codegen::BodyCodegen {
//         let name = self.generic_name(name, &generics);
//         codegen::Codegen::new(self, name)
//     }
// }

/// Adds all symbols this type uses into `dependencies`
fn type_dependencies(backend: &Backend, ty: &orco::Type, dependencies: &mut Vec<orco::Symbol>) {
    match ty {
        orco::Type::Symbol(name, generics) => {
            dependencies.push(backend.generic_name(*name, generics))
        }
        orco::Type::Array(ty, sz) if *sz > 0 => type_dependencies(backend, ty, dependencies),
        orco::Type::Struct { fields } => {
            for (_, ty) in fields {
                type_dependencies(backend, ty, dependencies);
            }
        }
        _ => (),
    }
}

impl std::fmt::Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "#include <stdint.h>")?;
        writeln!(f, "#include <stddef.h>")?;
        writeln!(f, "#include <stdbool.h>")?;
        writeln!(f)?;

        use std::collections::HashMap;
        #[derive(Default)]
        struct TopSorter {
            deps: HashMap<orco::Symbol, Vec<orco::Symbol>>,
            /// If name isn't present - not visited,
            /// otherwise stores whether it has finished processing
            /// (if false is encountered, loop is detected)
            visited: HashMap<orco::Symbol, bool>,
            order: Vec<orco::Symbol>,
        }

        let types = self.types.pin();
        let mut sorter = TopSorter::default();
        for (name, ty) in types.into_iter() {
            let mut dependencies = Vec::new();
            type_dependencies(self, ty, &mut dependencies);
            sorter.deps.insert(*name, dependencies);

            if matches!(ty, orco::Type::Struct { .. }) {
                let name = self.cname(*name);
                writeln!(f, "typedef struct {name} {name};")?;
            }
        }

        fn topsort(name: orco::Symbol, sorter: &mut TopSorter) {
            use std::collections::hash_map::Entry;
            match sorter.visited.entry(name) {
                Entry::Occupied(finished) => {
                    if *finished.get() {
                        return;
                    }
                    panic!(
                        "type dependency cycle detected on {name}, possibly an infinitely-recursive type",
                    );
                }
                Entry::Vacant(entry) => entry.insert(false),
            };

            let deps = sorter.deps.remove(&name);
            for dep in deps.into_iter().flat_map(Vec::into_iter) {
                topsort(dep, sorter);
            }

            sorter.visited.insert(name, true);
            sorter.order.push(name);
        }

        for (name, _) in types.into_iter() {
            topsort(*name, &mut sorter);
        }
        writeln!(f)?;

        for name in sorter.order {
            let Some(ty) = types.get(&name) else {
                continue;
            };

            writeln!(
                f,
                "typedef {};",
                FmtType {
                    backend: self,
                    ty: &ty,
                    constant: false,
                    name: Some(&self.cname(name))
                }
            )?;
        }

        writeln!(f)?;

        for (name, signature) in self.functions.pin().into_iter() {
            writeln!(
                f,
                "{};",
                symbols::FmtFunction {
                    backend: self,
                    name: &self.cname(*name),
                    signature,
                    name_all_args: false,
                }
            )?;
        }

        writeln!(f)?;

        for def in self.definitions.lock().unwrap().iter() {
            writeln!(f, "{def}\n")?;
        }

        Ok(())
    }
}
