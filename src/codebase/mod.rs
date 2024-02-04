use super::*;
use std::sync::Mutex;
pub use string_interner::StringInterner;

/// A symbol inside of a string interner, see [`Codebase::interner`]
pub type Symbol = string_interner::DefaultSymbol;

/// Everything related to files in the codebase
pub mod file;
pub use file::*;

/// Codebase, holds string interner, all the source files, all compilation units and handles
/// diagnostics
pub struct Codebase {
    /// String interner, see [`StringInterner`]
    pub interner: Mutex<StringInterner>,
    /// Compilation units
    pub units: std::collections::HashMap<Symbol, Mutex<Box<dyn Unit + Send>>>,
    /// Diagnostic writer, used to configure, how diagnostics are rendered
    pub diagnostic_writer: diagnostic::DiagnosticWriter,
    files: std::sync::Mutex<Vec<File>>,
}

impl Codebase {
    /// Create a new codebase, use this if you want to specify a custom diagnostic renderer.
    /// Otherwise use [`Codebase::default`]
    pub fn new(diagnostic_writer: diagnostic::DiagnosticWriter) -> Self {
        Self {
            interner: Mutex::new(StringInterner::new()),
            files: std::sync::Mutex::new(Vec::new()),
            diagnostic_writer,
            units: std::collections::HashMap::new(),
        }
    }

    /// Intern a symbol or (if exists) get it's interned version
    pub fn interned(&self, symbol: impl AsRef<str>) -> Symbol {
        self.interner.lock().unwrap().get_or_intern(symbol)
    }

    /// Resolve a symbol to a string
    pub fn resolve_symbol(&self, symbol: Symbol) -> String {
        if let Some(symbol) = self.interner.lock().unwrap().resolve(symbol) {
            symbol.to_owned()
        } else {
            use string_interner::Symbol;
            self.report(
                Diagnostic::bug()
                    .with_message("Failed to resolve symbol")
                    .with_notes(vec![format!("Symbol: {}", symbol.to_usize())]),
            );
            "<error>".to_owned()
        }
    }
}

impl Default for Codebase {
    fn default() -> Self {
        Self::new(diagnostic::DiagnosticWriter::default())
    }
}

impl std::fmt::Debug for Codebase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Codebase")
            .field(
                "units",
                &self
                    .units
                    .keys()
                    .map(|unit| self.resolve_symbol(*unit))
                    .collect::<Vec<_>>(),
            )
            .field("diagnostic_writer", &self.diagnostic_writer)
            .field("files", &self.files)
            .finish()
    }
}
