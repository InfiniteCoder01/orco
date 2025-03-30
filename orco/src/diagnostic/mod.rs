pub use miette;
pub use thiserror;

pub use miette::Diagnostic as Miette;
pub use miette::SourceSpan as Span;

use miette::LabeledSpan;
use std::sync::Mutex;
use thiserror::Error;

/// See [SyntaxError]
pub mod syntax_error;
pub use syntax_error::SyntaxError;

/// OrCo diagnostic trait
pub trait Diagnostic: Miette + Send + Sync {}

/// Source file
pub type SourceFile = std::sync::Arc<miette::NamedSource<String>>;

/// Diagnostic Context - used to emit diagnostics.
/// Thread safe and can be passed immutable to add diagnostic
#[derive(Default)]
pub struct DiagCtx {
    current_file: Option<SourceFile>,
    diagnostics: Mutex<Vec<Box<dyn Diagnostic>>>,
}

impl std::fmt::Debug for DiagCtx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let diagnostics = self.diagnostics.lock().unwrap();
        f.debug_struct("DiagCtx")
            .field("file", &self.current_file)
            .field(
                "diagnostics",
                &format!(
                    "<{} diagnostic{}>",
                    diagnostics.len(),
                    if diagnostics.len() == 1 { "" } else { "s" }
                ),
            )
            .finish()
    }
}

impl DiagCtx {
    /// Creates a new diagnostic context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set source file
    pub fn set_source(&mut self, file: SourceFile) {
        self.current_file = Some(file);
    }

    /// Emit a boxed diagnostic, returning a builder for it.
    /// Mostly used internally
    pub fn diagnostic<T: Diagnostic + 'static>(&self, diagnostic: Box<T>) -> DiagnosticBuilder<T> {
        DiagnosticBuilder(Some(diagnostic), self)
    }

    /// Emit a syntax error
    pub fn syntax_error(&self, message: impl ToString) -> DiagnosticBuilder<SyntaxError> {
        self.diagnostic(Box::new(SyntaxError {
            src: self.current_file.clone(),
            message: message.to_string(),
            labels: Vec::new(),
        }))
    }

    /// Emit all stored diagnostics
    pub fn emit(self) {
        for diagnostic in self.diagnostics.into_inner().unwrap() {
            println!("{:?}", miette::Report::new_boxed(diagnostic));
        }
    }
}

/// Diagnostic builder, used to give mutable access to
/// the diagnostic and then add it to [DiagCtx]
pub struct DiagnosticBuilder<'a, T: Diagnostic + 'static>(Option<Box<T>>, &'a DiagCtx);

impl<T: Diagnostic + 'static> std::ops::Deref for DiagnosticBuilder<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

impl<T: Diagnostic + 'static> std::ops::DerefMut for DiagnosticBuilder<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap()
    }
}

impl<T: Diagnostic + 'static> std::ops::Drop for DiagnosticBuilder<'_, T> {
    fn drop(&mut self) {
        self.1
            .diagnostics
            .lock()
            .unwrap()
            .push(self.0.take().unwrap())
    }
}
