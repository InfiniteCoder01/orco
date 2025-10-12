//! C transpilation backend for orco
//! See [Backend]
#![warn(missing_docs)]

/// Code generation, used to generate function bodies.
pub mod codegen;
/// Declaration, enough to create headers
pub mod declare;
pub use declare::{Declaration, DeclarationKind};

/// Central declaration & codegen backend
#[derive(Debug, Default)]
pub struct Backend {
    /// A map from symbol to a declaration
    pub decls: scc::HashMap<orco::Symbol, Declaration>,
    /// Al definitions, in no particular order
    /// TODO: Unordered container would work better
    pub defs: scc::Stack<String>,
}

impl Backend {
    #[allow(missing_docs)]
    pub fn new() -> Self {
        Self::default()
    }
}

impl std::fmt::Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "#include <stdint.h>")?;
        writeln!(f, "#include <stddef.h>")?;
        writeln!(f, "#include <stdbool.h>")?;
        writeln!(f)?;

        let mut result = Ok(());
        self.decls.iter_sync(|_, decl| {
            result = writeln!(f, "{decl}");
            result.is_ok()
        });
        result?;

        writeln!(f).unwrap();
        let guard = scc::Guard::new();
        for def in self.defs.iter(&guard) {
            writeln!(f, "{def}\n")?;
        }
        Ok(())
    }
}

/// Escape the symbol to be a valid C identifier. Possibly does mangling
pub fn escape(symbol: orco::Symbol) -> String {
    symbol
        .as_str()
        .replace("::", "_")
        .replace(['.', ':', '/', '-'], "_")
}
