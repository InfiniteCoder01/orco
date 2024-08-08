pub use crate::source::*;
pub use ariadne::{Color, ReportKind};

pub use miette::{Diagnostic, NamedSource, SourceSpan};
pub use thiserror::Error;

/// Color conventions
#[allow(non_upper_case_globals)]
pub mod colors {
    use super::Color;
    /// A color for a regular label
    pub const Label: Color = Color::Blue;
    /// A color for expected part of "expected ..., got ..." error message
    pub const Expected: Color = Color::Red;
    /// A color for got part of "expected ..., got ..." error message
    pub const Got: Color = Label;
}

/// [`ariadne::Report`] with [`crate::Span`]
pub type Report = ariadne::Report<'static, Span>;

/// [`ariadne::Label`] with [`crate::Span`]
pub type Label = ariadne::Label<Span>;

impl ariadne::Span for Span {
    type SourceId = Src;

    fn source(&self) -> &Self::SourceId {
        &self.0
    }

    fn start(&self) -> usize {
        self.1.start
    }

    fn end(&self) -> usize {
        self.1.end
    }
}

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

/// Extensions for [`ariadne::ReportBuilder`]
pub trait ReportBuilderExt {
    /// Add an optional label
    fn opt_label(self, span: Option<Span>, label: impl FnOnce(Label) -> Label) -> Self;
}

impl ReportBuilderExt for ariadne::ReportBuilder<'_, Span> {
    fn opt_label(self, span: Option<Span>, label: impl FnOnce(Label) -> Label) -> Self {
        self.with_labels(span.map(|span| label(Label::new(span))))
    }
}

// ------------------------------------------ Reporters

/// Error reporter
pub trait ErrorReporter {
    /// Report an error
    fn report(&mut self, report: Report);

    /// Report an error
    fn report_miette(&mut self, report: miette::Report);
}

impl ErrorReporter for Vec<miette::Report> {
    fn report(&mut self, _report: Report) {}

    fn report_miette(&mut self, report: miette::Report) {
        self.push(report);
    }
}

/// Default error reporter
#[derive(Clone, Debug, Default)]
pub struct DefaultReporter;

impl ErrorReporter for DefaultReporter {
    fn report(&mut self, report: Report) {
        struct Source(Src);
        impl AsRef<str> for Source {
            fn as_ref(&self) -> &str {
                self.0.content()
            }
        }

        if let Err(err) = report.eprint(ariadne::FnCache::new(|id: &Src| Ok(Source(id.clone())))) {
            log::error!("Failed to render diagnostic report: {}", err);
        }
    }

    fn report_miette(&mut self, report: miette::Report) {
        eprintln!("{:?}", report);
    }
}
