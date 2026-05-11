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
pub use symbols::SymbolKind;

/// Code generation, used to generate function bodies.
pub mod codegen;
pub use codegen::Codegen;

/// Root backend struct
#[derive(Debug, Default)]
pub struct Backend<'a> {
    /// A map from symbol to it's declaration
    pub symbols: scc::HashMap<orco::Symbol, SymbolKind>,
    /// Interned types
    interned: scc::HashSet<orco::Symbol>,
    /// Definitions
    definitions: scc::Stack<String>,
    /// The default macro handler
    pub macros: orco::impls::MacroServer<'a>,
}

impl Backend<'_> {
    #[allow(missing_docs)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Declares a symbol
    pub fn symbol(&self, name: orco::Symbol, kind: SymbolKind) {
        self.symbols
            .insert_sync(name, kind)
            .unwrap_or_else(|_| panic!("symbol {name:?} is already declared"))
    }

    /// Returns a previously declared symbol
    pub fn get_symbol(
        &self,
        name: orco::Symbol,
    ) -> scc::hash_map::OccupiedEntry<'_, orco::Symbol, SymbolKind> {
        self.symbols
            .get_sync(&name)
            .unwrap_or_else(|| panic!("undeclared symbol {name}"))
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
                let symbol = self.get_symbol(symbol);
                match symbol.get() {
                    SymbolKind::Type(ty)
                        if inline_struct || !matches!(ty, orco::Type::Struct { .. }) =>
                    {
                        self.inline_type_aliases(ty.clone(), inline_struct)
                    }
                    _ => ty,
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
        self.symbol(
            name,
            SymbolKind::Function(orco::types::FunctionSignature {
                attrs,
                params,
                return_type,
            }),
        );
    }

    fn type_(&self, name: orco::Symbol, mut ty: orco::Type) {
        self.intern_type(&mut ty, true);
        self.symbol(name, SymbolKind::Type(ty));
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

impl std::fmt::Display for Backend<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "#include <stdint.h>")?;
        writeln!(f, "#include <stddef.h>")?;
        writeln!(f, "#include <stdbool.h>")?;
        writeln!(f)?;

        let mut result = Ok(());
        self.symbols.iter_sync(|name, sym| {
            let sym = format!(
                "{}",
                symbols::FmtSymbol {
                    name: &crate::symname(*name),
                    kind: sym,
                }
            );
            result = writeln!(
                f,
                "{}{}",
                sym,
                if sym.lines().count() > 1 { "\n" } else { "" }
            );
            result.is_ok()
        });
        result?;

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
