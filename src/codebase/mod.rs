use super::*;
pub use string_interner::StringInterner;
/// A symbol inside of a string interner, see [`Codebase::interner`]
pub type Symbol = string_interner::DefaultSymbol;

/// Everything related to files in the codebase
pub mod file;
pub use file::*;

/// Codebase, holds string interner, all the source files, all compilation units and handles
/// diagnostics
pub struct Codebase<'a> {
    /// String interner, see [`StringInterner`]
    pub interner: StringInterner,
    /// Compilation units
    pub units: std::collections::HashMap<Symbol, Box<dyn Unit>>,
    /// Diagnostic writer, used to configure, how diagnostics are rendered
    pub diagnostic_writer: diagnostic::DiagnosticWriter,
    files: Vec<File<'a>>,
}

impl<'a> Codebase<'a> {
    /// Create a new codebase, use this if you want to specify a custom diagnostic renderer.
    /// Otherwise use [`Codebase::default`]
    pub fn new(diagnostic_writer: diagnostic::DiagnosticWriter) -> Self {
        Self {
            interner: StringInterner::new(),
            files: Vec::new(),
            diagnostic_writer,
            units: std::collections::HashMap::new(),
        }
    }
}

impl<'a> Default for Codebase<'a> {
    fn default() -> Self {
        Self::new(diagnostic::DiagnosticWriter::default())
    }
}

impl<'a> std::fmt::Debug for Codebase<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Codebase")
            .field(
                "units",
                &self
                    .units
                    .keys()
                    .map(|unit| {
                        self.interner
                            .resolve(*unit)
                            .expect("failed to resolve unit name")
                    })
                    .collect::<Vec<_>>(),
            )
            .field("diagnostic_writer", &self.diagnostic_writer)
            .field("files", &self.files)
            .finish()
    }
}
