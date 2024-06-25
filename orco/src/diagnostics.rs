pub use crate::source::*;
pub use ariadne::{ColorGenerator, Label, ReportKind};
pub use miette::{Diagnostic, NamedSource, SourceSpan};
pub use thiserror::Error;

/// Diagnostic report (error, warning, etc.)
pub type Report = ariadne::Report<'static, Span>;

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
        self.content().read_span(span, context_lines_before, context_lines_after)
    }
}

impl Span {
    pub fn named_source(&self) -> NamedSource<Src> {
        NamedSource::new(self.0.path().to_string_lossy(), self.0.clone())
    }

    pub fn source_span(&self) -> SourceSpan {
        self.1.clone().into()
    }
}

/// Error reporter
pub trait ErrorReporter {
    /// Report an error
    fn report(&mut self, report: Report);

    /// Report a type error (an error with a given message, a span of the error, and maybe some
    /// labels)
    fn report_type_error(
        &mut self,
        message: String,
        r#where: Span,
        labels: Vec<(&'static str, Span)>,
    ) {
        let mut colors = ColorGenerator::new();
        let report = Report::build(ReportKind::Error, r#where.0.clone(), r#where.1.start)
            .with_message(message)
            .with_label(
                Label::new(r#where)
                    .with_message("Here")
                    .with_color(colors.next()),
            )
            .with_labels(labels.into_iter().map(|(label, span)| {
                Label::new(span)
                    .with_message(label)
                    .with_color(colors.next())
            }));
        self.report(report.finish());
    }

    fn report_miette(&mut self, error: miette::Report) {
        eprintln!("{:?}", error);
    }

    /// Check if there were any errors
    fn has_errors(&self) -> bool;
}

impl ErrorReporter for Vec<Report> {
    fn report(&mut self, report: Report) {
        self.push(report);
    }

    fn has_errors(&self) -> bool {
        !self.is_empty()
    }
}

/// Default error reporter
#[derive(Clone, Debug, Default)]
pub struct DefaultReporter(usize);

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

        self.0 += 1;
    }

    fn has_errors(&self) -> bool {
        self.0 > 0
    }
}
