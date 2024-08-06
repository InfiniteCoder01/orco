pub use crate::source::*;
pub use ariadne::{ColorGenerator, Label, ReportKind};
pub use miette::{Diagnostic, NamedSource, SourceSpan};
pub use thiserror::Error;

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

/// Error reporter
pub trait ErrorReporter {
    /// Report an error
    fn report(&mut self, report: miette::Report);

    /// Report an error, old ariadne API, getting gradually transitioned to [`ErrorReporter::report`]
    fn report_ariadne(&mut self, report: ariadne::Report<'static, Span>);

    /// Report a type error (an error with a given message, a span of the error, and maybe some
    /// labels)
    /// Old ariadne API, getting gradually transitioned to [`ErrorReporter::report`]
    fn report_type_error(
        &mut self,
        message: String,
        r#where: Span,
        labels: Vec<(&'static str, Span)>,
    ) {
        let mut colors = ColorGenerator::new();
        let report = ariadne::Report::build(ReportKind::Error, r#where.0.clone(), r#where.1.start)
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
        self.report_ariadne(report.finish());
    }
}

impl ErrorReporter for Vec<miette::Report> {
    fn report(&mut self, report: miette::Report) {
        self.push(report);
    }

    fn report_ariadne(&mut self, _report: ariadne::Report<'static, Span>) {}
}

/// Default error reporter
#[derive(Clone, Debug, Default)]
pub struct DefaultReporter;

impl ErrorReporter for DefaultReporter {
    fn report(&mut self, report: miette::Report) {
        eprintln!("{:?}", report);
    }

    fn report_ariadne(&mut self, report: ariadne::Report<'static, Span>) {
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
}
