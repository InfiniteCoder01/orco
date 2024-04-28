/// Span (holds both span and a file path)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Span(pub Src, pub std::ops::Range<usize>);

impl Span {
    /// Extend span up to the end of another span
    pub fn extend(&self, span: &Span) -> Span {
        assert_eq!(self.0, span.0);
        Span(
            self.0.clone(),
            self.1.start.min(span.1.start)..self.1.end.max(span.1.end),
        )
    }
}

/// Source (one source file)
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Source(std::path::PathBuf, String);

impl Source {
    /// Path
    pub fn path(&self) -> &std::path::Path {
        &self.0
    }

    /// Content
    pub fn content(&self) -> &str {
        &self.1
    }
}

impl std::ops::Deref for Source {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

/// Source ID
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Src(std::sync::Arc<Source>);

impl Src {
    /// Create a new source
    pub fn new(content: String, path: std::path::PathBuf) -> Self {
        Self(std::sync::Arc::new(Source(path, content)))
    }

    /// Load a source
    pub fn load(path: std::path::PathBuf) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(&path)?;
        Ok(Self(std::sync::Arc::new(Source(path, content))))
    }
}

impl std::ops::Deref for Src {
    type Target = std::sync::Arc<Source>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for Src {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.path().display())
    }
}
