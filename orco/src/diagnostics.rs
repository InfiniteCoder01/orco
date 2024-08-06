pub use crate::source::*;
pub use miette::{Diagnostic, NamedSource, SourceSpan};
pub use thiserror::Error;

impl miette::SourceCode for Src {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn miette::SpanContents<'a> + 'a>, miette::MietteError> {
        self.content()
            .read_span(span, context_lines_before, context_lines_after)
    }
}

impl Span {
    /// Get a [`miette::NamedSource`] out of this span
    pub fn named_source(&self) -> NamedSource<Src> {
        NamedSource::new(self.0.path().to_string_lossy(), self.0.clone())
    }

    /// Get a [`miette::SourceSpan`] out of this span
    pub fn source_span(&self) -> SourceSpan {
        self.1.clone().into()
    }
}

/// Error reporter
pub trait ErrorReporter {
    /// Report an error
    fn report(&mut self, report: miette::Report);
}

impl ErrorReporter for Vec<miette::Report> {
    fn report(&mut self, report: miette::Report) {
        self.push(report);
    }
}

/// Default error reporter
#[derive(Clone, Debug, Default)]
pub struct DefaultReporter;

impl ErrorReporter for DefaultReporter {
    fn report(&mut self, report: miette::Report) {
        eprintln!("{:?}", report);
    }
}
