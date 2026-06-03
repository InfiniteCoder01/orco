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

/// Symbol container types
pub mod symbols;

/// Code generation, used to generate function bodies.
pub mod codegen;
pub use codegen::Codegen;

/// Root backend struct
#[derive(Debug, Default)]
pub struct Backend<'a> {
    /// Type aliases
    pub types: scc::HashMap<orco::Symbol, orco::Type>,
    /// Function declarations
    pub functions: scc::HashMap<orco::Symbol, orco::types::FunctionSignature>,
    /// Definitions
    definitions: scc::Stack<String>,
    /// Interned types
    interned: scc::HashSet<orco::Symbol>,
    /// The default macro handler
    pub macros: orco::impls::MacroServer<'a>,
}

impl Backend<'_> {
    #[allow(missing_docs)]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a definition
    pub fn define(&self, code: String) {
        self.definitions.push(code);
    }

    /// If ty is a type alias (but not a struct), inlines it.
    /// Does not inline inner types
    pub fn inline_type_aliases(&self, ty: orco::Type, inline_struct: bool) -> orco::Type {
        match ty {
            orco::Type::Symbol(symbol) => {
                let symbol = self
                    .types
                    .get_sync(&symbol)
                    .unwrap_or_else(|| panic!("undeclared type {symbol}"));
                if inline_struct || !matches!(*symbol, orco::Type::Struct { .. }) {
                    self.inline_type_aliases(symbol.clone(), inline_struct)
                } else {
                    ty
                }
            }
            ty => ty,
        }
    }

    /// Intern the following type and it's insides.
    pub fn intern_type(&self, ty: &mut orco::Type, named: bool) {
        match ty {
            orco::Type::Array(ty, _) => {
                self.intern_type(ty.as_mut(), false) // TODO: More work on arrays
            }
            orco::Type::Struct { fields } if named => {
                for (_, ty) in fields {
                    self.intern_type(ty, false);
                }
            }
            orco::Type::Struct { fields } if !named => {
                let sym = orco::Symbol::new(&format!("s {}", ty.hashable_name()));
                let ty = std::mem::replace(ty, orco::Type::Symbol(sym));
                if self.interned.insert_sync(sym).is_ok() {
                    use orco::DeclarationBackend as _;
                    self.type_(sym, ty);
                }
            }
            _ => (),
        }
    }
}

impl<'a> orco::DeclarationBackend<'a> for Backend<'a> {
    fn function(
        &self,
        name: orco::Symbol,
        mut params: Vec<(Option<String>, orco::Type)>,
        mut return_type: Option<orco::Type>,
        attrs: orco::attrs::FunctionAttributes,
    ) {
        for (_, ty) in &mut params {
            self.intern_type(ty, false);
        }
        if let Some(rt) = &mut return_type {
            self.intern_type(rt, false);
        }
        self.functions
            .insert_sync(
                name,
                orco::types::FunctionSignature {
                    params,
                    return_type,
                    attrs,
                },
            )
            .unwrap_or_else(|_| panic!("function {name} is already declared"))
    }

    fn type_(&self, name: orco::Symbol, mut ty: orco::Type) {
        self.intern_type(&mut ty, true);
        self.types
            .insert_sync(name, ty)
            .unwrap_or_else(|_| panic!("type {name} is already declared"))
    }

    fn macro_(
        &self,
        name: orco::Symbol,
        callback: impl Fn(&[orco::Type]) + Send + Sync + 'a,
        call_once: bool,
    ) {
        self.macros.macro_(name, callback, call_once)
    }

    fn invoke_macro(&self, name: orco::Symbol, args: &[orco::Type]) {
        self.macros.invoke_macro(name, args);
    }
}

impl orco::CodegenBackend for crate::Backend<'_> {
    fn function(&self, name: orco::Symbol) -> impl orco::codegen::BodyCodegen {
        codegen::Codegen::new(self, name)
    }
}

/// Adds all symbols this type uses into `dependencies`
pub fn type_dependencies(ty: &orco::Type, dependencies: &mut Vec<orco::Symbol>) {
    match ty {
        orco::Type::Symbol(name) => dependencies.push(*name),
        orco::Type::Array(ty, sz) if *sz > 0 => type_dependencies(ty, dependencies),
        orco::Type::Struct { fields } => {
            for (_, ty) in fields {
                type_dependencies(ty, dependencies);
            }
        }
        _ => (),
    }
}

impl std::fmt::Display for Backend<'_> {
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

        let mut sorter = TopSorter::default();
        let mut result = Ok(());
        self.types.iter_sync(|name, ty| {
            let mut dependencies = Vec::new();
            type_dependencies(ty, &mut dependencies);
            sorter.deps.insert(*name, dependencies);
            if matches!(ty, orco::Type::Struct { .. }) {
                let name = symname(*name);
                result = writeln!(f, "typedef struct {name}_struct {name};");
            }

            result.is_ok()
        });
        result?;

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

        self.types.iter_sync(|name, _| {
            topsort(*name, &mut sorter);
            true
        });
        writeln!(f)?;

        for name in sorter.order {
            let Some(ty) = self.types.get_sync(&name) else {
                continue;
            };

            writeln!(
                f,
                "typedef {};",
                FmtType {
                    ty: &ty,
                    constant: false,
                    name: Some(&symname(name))
                }
            )?;
        }

        writeln!(f)?;

        self.functions.iter_sync(|name, signature| {
            result = writeln!(
                f,
                "{};",
                symbols::FmtFunction {
                    name: &symname(*name),
                    signature,
                    name_all_args: false,
                }
            );

            result.is_ok()
        });
        result?;

        writeln!(f)?;

        for def in self.definitions.iter(&scc::Guard::new()) {
            writeln!(f, "{def}\n")?;
        }

        Ok(())
    }
}

/// Get the name of the symbol used in generated code
fn symname(symbol: orco::Symbol) -> String {
    // TODO: Needs work

    // Take only the method name, not the path
    // FIXME: conflicts...
    let symbol = &symbol[symbol.rfind([':', '.']).map_or(0, |i| i + 1)..];

    let mut symbol = symbol.replace(|c: char| !c.is_ascii_alphanumeric(), "_");
    if symbol.chars().next().is_none_or(|c| c.is_ascii_digit()) {
        symbol.insert(0, '_');
    }

    symbol
}
