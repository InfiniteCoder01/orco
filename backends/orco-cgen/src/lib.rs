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
pub mod codegen;

/// Central declaration & codegen backend
#[derive(Debug, Default)]
pub struct Backend {
    /// A map from symbol to it's definition
    pub symbols: scc::HashMap<orco::Symbol, SymbolKind>,
    /// All function defs, in no particular order
    // TODO: Unordered container would work better
    pub defs: scc::Stack<String>,
    type_interner: orco::type_intern::TypeInterner,
}

impl Backend {
    #[allow(missing_docs)]
    pub fn new() -> Backend {
        Self::default()
    }
}

impl orco::Backend for Backend {
    fn function(
        &self,
        name: orco::Symbol,
        mut params: Vec<(Option<orco::Symbol>, orco::Type)>,
        mut return_type: orco::Type,
    ) -> impl orco::codegen::BodyCodegen<'_> {
        for (_, ty) in &mut params {
            self.type_interner.on_type(self, ty, false, None);
        }
        self.type_interner
            .on_type(self, &mut return_type, false, Some("void".into()));

        let sig = symbols::FunctionSignature {
            params,
            return_type,
        };

        let codegen = codegen::function(self, name, &sig);
        self.symbols
            .entry_sync(name)
            .and_modify(|_| panic!("symbol {name:?} is already declared"))
            .or_insert(SymbolKind::Function(sig));

        codegen
    }

    fn type_(&self, name: orco::Symbol, mut ty: orco::Type) {
        self.type_interner
            .on_type(self, &mut ty, true, Some("void".into()));
        self.symbols
            .entry_sync(name)
            .and_modify(|_| panic!("symbol {name:?} is already declared"))
            .or_insert(SymbolKind::Type(ty));
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
            let sym = format!("{}", symbols::FmtSymbol(*name, sym));
            result = writeln!(
                f,
                "{}{}",
                sym,
                if sym.lines().count() > 1 { "\n" } else { "" }
            );
            result.is_ok()
        });
        result?;

        writeln!(f).unwrap();
        let guard = scc::Guard::new();
        for def in self.defs.iter(&guard) {
            writeln!(f, "{def}\n")?;
        }

        result
    }
}

/// Escape the symbol to be a valid C identifier. Possibly does mangling
pub fn escape(symbol: orco::Symbol) -> String {
    let symbol = symbol
        .as_str()
        .replace("::", "_")
        .replace(['.', ':', '/', '-', ' '], "_");
    if symbol.chars().next().unwrap().is_digit(10) {
        format!("_{symbol}")
    } else {
        symbol
    }
}
