use super::*;
pub use string_interner::StringInterner;
pub type Symbol = string_interner::DefaultSymbol;

pub mod file;
pub use file::*;

pub struct Codebase<'a> {
    pub interner: StringInterner,
    pub units: std::collections::HashMap<Symbol, Box<dyn Unit>>,
    pub diagnostic_writer: diagnostic::DiagnosticWriter,
    files: Vec<File<'a>>,
}

impl<'a> Codebase<'a> {
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
