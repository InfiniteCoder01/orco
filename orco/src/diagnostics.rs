pub use crate::{Span, Src};
pub use ariadne::{ColorGenerator, Label, ReportKind};

/// Diagnostic report (error, warning, etc.)
pub type Report<'a> = ariadne::Report<'a, Span>;

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

/// Error reporter
pub trait ErrorReporter {
    /// Report an error
    fn report(&mut self, report: Report);
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
