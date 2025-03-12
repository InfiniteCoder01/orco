use miette::Diagnostic as Miette;
use miette::LabeledSpan;
use thiserror::Error;

pub mod syntax_error;
pub use syntax_error::SyntaxError;

pub trait Diagnostic: Miette + Send + Sync {}
pub type SourceFile = std::sync::Arc<miette::NamedSource<String>>;

#[derive(Default)]
pub struct DiagCtx {
    current_file: Option<SourceFile>,
    diagnostics: Vec<Box<dyn Diagnostic>>,
}

impl DiagCtx {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_source(&mut self, file: SourceFile) {
        self.current_file = Some(file);
    }

    pub fn diagnostic<T: Diagnostic + 'static>(&mut self, mut diagnostic: Box<T>) -> &mut T {
        let ptr = diagnostic.as_mut() as *mut T;
        self.diagnostics.push(diagnostic);
        unsafe { &mut *ptr }
    }

    pub fn syntax_error(&mut self, message: impl ToString) -> &mut SyntaxError {
        self.diagnostic(Box::new(SyntaxError {
            src: self.current_file.clone(),
            message: message.to_string(),
            labels: Vec::new(),
        }))
    }

    pub fn emit(self) {
        for diagnostic in self.diagnostics {
            // println!("{}", DiagnosticFmt(diagnostic.as_ref(), &report_handler));
            println!("{:?}", miette::Report::new_boxed(diagnostic));
        }
    }
}
