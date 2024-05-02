pub use crate::{Span, Src};
pub use ariadne::{ColorGenerator, Label, ReportKind};

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

/// Spanned value (uses [Span])
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Spanned<T> {
    /// Inner value
    pub inner: T,
    /// Span
    pub span: Span,
}

impl<T> std::ops::Deref for Spanned<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> std::ops::DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T> Spanned<T> {
    /// Map the inner value
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Spanned<U> {
        Spanned {
            inner: f(self.inner),
            span: self.span,
        }
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
}

impl ErrorReporter for Vec<Report> {
    fn report(&mut self, report: Report) {
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
}
