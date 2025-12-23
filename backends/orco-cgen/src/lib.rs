//! C transpilation backend for orco.
//! Also used to generate C headers and is generally the reference for other backends
//! See [Backend]
// TODO: ABI
// TODO: Extra type interning
#![warn(missing_docs)]

/// Type formatting & other things
pub mod types;
use types::FmtType;

/// Symbol container types
pub mod symbols;
pub use symbols::SymbolKind;

/// Code generation, used to generate function bodies.
pub(crate) mod codegen;

/// Generics wrapper
pub(crate) mod generics;

/// Backend context. Either [Backend] or [`generics::Wrapper`]
pub trait BackendContext {
    /// Get the original backend
    fn backend(&self) -> &Backend;

    /// Defines a symbol
    fn symbol(&self, name: orco::Symbol, kind: SymbolKind);

    /// Escape the symbol to be a valid C identifier. Possibly does mangling.
    /// Substitutes # for either token pasting (`##_##`) or underscores (`_`), depending on the context
    fn escape(&self, symbol: orco::Symbol) -> String;

    /// Intern the following type and it's insides
    fn intern_type(&self, ty: &mut orco::Type, named: bool, replace_unit: bool);
}

/// Root backend struct
#[derive(Debug, Default)]
pub struct Backend {
    /// A map from symbol to it's definition
    pub symbols: scc::HashMap<orco::Symbol, SymbolKind>,
    /// Interned types
    interned: scc::HashSet<orco::Symbol>,
}

impl Backend {
    #[allow(missing_docs)]
    pub fn new() -> Backend {
        Self::default()
    }
}

impl BackendContext for Backend {
    fn backend(&self) -> &Backend {
        self
    }

    fn symbol(&self, name: orco::Symbol, kind: SymbolKind) {
        self.symbols
            .entry_sync(name)
            .and_modify(|_| panic!("symbol {name:?} is already declared"))
            .or_insert(kind);
    }

    fn escape(&self, symbol: orco::Symbol) -> String {
        escape(&symbol.replace('#', "_"))
    }

    fn intern_type(&self, ty: &mut orco::Type, named: bool, replace_unit: bool) {
        match ty {
            orco::Type::Array(ty, _) => self.intern_type(ty.as_mut(), false, false), // TODO: More work on arrays
            orco::Type::Struct(fields) if named => {
                for (_, ty) in fields {
                    self.intern_type(ty, false, false);
                }
            }
            orco::Type::Struct(fields) if !named => {
                if fields.is_empty() && replace_unit {
                    *ty = orco::Type::Symbol("void".into());
                    return;
                }
                let sym = orco::Symbol::new(&format!("s {}", ty.hashable_name()));
                let ty = std::mem::replace(ty, orco::Type::Symbol(sym));
                if self.interned.insert_sync(sym).is_ok() {
                    use orco::Backend as _;
                    self.type_(sym, ty);
                }
            }
            _ => (),
        }
    }
}

impl orco::Backend for Backend {
    fn function(
        &self,
        name: orco::Symbol,
        mut params: Vec<(Option<orco::Symbol>, orco::Type)>,
        mut return_type: orco::Type,
    ) -> impl orco::codegen::BodyCodegen {
        for (_, ty) in &mut params {
            self.intern_type(ty, false, false);
        }
        self.intern_type(&mut return_type, false, true);

        codegen::Codegen::new(
            self,
            name,
            symbols::FunctionSignature {
                params,
                return_type,
            },
        )
    }

    fn type_(&self, name: orco::Symbol, mut ty: orco::Type) {
        self.intern_type(&mut ty, true, false);
        self.symbol(name, SymbolKind::Type(ty));
    }

    fn generic(&self, params: Vec<orco::Symbol>) -> impl orco::Backend {
        generics::Wrapper {
            backend: self,
            params,
        }
    }
}

impl std::fmt::Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "#include <stdint.h>")?;
        writeln!(f, "#include <stddef.h>")?;
        writeln!(f, "#include <stdbool.h>")?;
        writeln!(f)?;

        let mut result = Ok(());
        self.symbols.iter_sync(|name, sym| {
            let sym = format!("{}", symbols::FmtSymbol(self, &self.escape(*name), sym));
            result = writeln!(
                f,
                "{}{}",
                sym,
                if sym.lines().count() > 1 { "\n" } else { "" }
            );
            result.is_ok()
        });

        result
    }
}

/// Escape the symbol to be a valid C identifier.
pub fn escape(symbol: &str) -> String {
    // Take only the method name, not the path
    // FIXME: Temproary, for better readability of generated code
    let symbol = &symbol[symbol.rfind(':').map_or(0, |i| i + 1)..];

    let mut symbol = symbol
        .replace("::", "_")
        .replace(|c: char| !c.is_ascii_alphanumeric(), "_");
    if symbol.chars().next().unwrap().is_ascii_digit() {
        symbol.insert(0, '_');
    }
    symbol
}
