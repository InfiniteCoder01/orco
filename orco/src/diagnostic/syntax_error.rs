use super::*;

/// A regular syntax error
#[derive(Clone, Debug, Error)]
#[error("{message}")]
pub struct SyntaxError {
    pub(super) src: Option<SourceFile>,
    pub(super) message: String,
    pub(super) labels: Vec<LabeledSpan>,
}

impl SyntaxError {
    pub fn label(
        &mut self,
        label: Option<impl ToString>,
        span: impl Into<miette::SourceSpan>,
    ) -> &mut Self {
        self.labels.push(LabeledSpan::new_with_span(
            label.map(|label| label.to_string()),
            span,
        ));
        self
    }
}

impl Miette for SyntaxError {
    fn severity(&self) -> Option<miette::Severity> {
        Some(miette::Severity::Error)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        Some(Box::new(self.labels.iter().cloned()))
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        self.src.as_ref().map(|src| src as _)
    }
}

impl Diagnostic for SyntaxError {}
