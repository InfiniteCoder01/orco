use super::*;

/// A generic "symbol not found" error
#[derive(Clone, Debug, Error)]
#[error("{symbol} could not be found")]
pub struct SymbolNotFound {
    pub(super) src: Option<SourceFile>,
    pub(super) symbol: String,
    pub(super) labels: Vec<LabeledSpan>,
}

impl SymbolNotFound {
    /// Add a spanned label.
    /// See also [`Self::mark`]
    pub fn label(
        &mut self,
        label: impl ToString,
        span: impl Into<miette::SourceSpan>,
    ) -> &mut Self {
        self.labels
            .push(LabeledSpan::new_with_span(Some(label.to_string()), span));
        self
    }

    /// Add a spanned label without text
    /// See also [`Self::label`]
    pub fn mark(&mut self, span: impl Into<miette::SourceSpan>) -> &mut Self {
        self.labels.push(LabeledSpan::new_with_span(None, span));
        self
    }
}

impl Miette for SymbolNotFound {
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

impl Diagnostic for SymbolNotFound {}
