pub use miette;
pub use thiserror;

pub use miette::Diagnostic as Miette;
pub use miette::SourceSpan as Span;

use miette::LabeledSpan;
use std::sync::Mutex;
use thiserror::Error;

pub mod syntax_error;
pub use syntax_error::SyntaxError;

pub trait Diagnostic: Miette + Send + Sync {}
pub type SourceFile = std::sync::Arc<miette::NamedSource<String>>;

#[derive(Default)]
pub struct DiagCtx {
    current_file: Option<SourceFile>,
    diagnostics: Mutex<Vec<Box<dyn Diagnostic>>>,
}

impl DiagCtx {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_source(&mut self, file: SourceFile) {
        self.current_file = Some(file);
    }

    pub fn diagnostic<T: Diagnostic + 'static>(&self, diagnostic: Box<T>) -> DiagnosticBuilder<T> {
        DiagnosticBuilder(Some(diagnostic), self)
    }

    pub fn syntax_error(&self, message: impl ToString) -> DiagnosticBuilder<SyntaxError> {
        self.diagnostic(Box::new(SyntaxError {
            src: self.current_file.clone(),
            message: message.to_string(),
            labels: Vec::new(),
        }))
    }

    pub fn emit(self) {
        for diagnostic in self.diagnostics.into_inner().unwrap() {
            // println!("{}", DiagnosticFmt(diagnostic.as_ref(), &report_handler));
            println!("{:?}", miette::Report::new_boxed(diagnostic));
        }
    }
}

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
