/// Span (holds both span and a file path)
#[derive(Clone, Debug)]
pub struct Span(pub Src, pub std::ops::Range<usize>);

impl Span {
    /// Create a new span, that only holds the string
    pub fn new(text: &str) -> Self {
        Self(
            Src::new(text.to_owned(), "arbitruary-string".into()),
            0..text.len(),
        )
    }

    /// Extend span up to the end of another span
    pub fn extend(&self, span: &Span) -> Span {
        assert_eq!(self.0, span.0);
        Span(
            self.0.clone(),
            self.1.start.min(span.1.start)..self.1.end.max(span.1.end),
        )
    }
}

impl AsRef<str> for Span {
    fn as_ref(&self) -> &str {
        &self.0[self.1.clone()]
    }
}

impl std::ops::Deref for Span {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl std::cmp::PartialEq for Span {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl std::cmp::Eq for Span {}

impl std::cmp::PartialOrd for Span {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for Span {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

impl std::hash::Hash for Span {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state)
    }
}

/// Spanned value (uses [Span])
#[derive(derivative::Derivative, Clone)]
#[derivative(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Spanned<T> {
    /// Inner value
    pub inner: T,
    /// Span
    #[derivative(
        Debug = "ignore",
        PartialEq = "ignore",
        PartialOrd = "ignore",
        Ord = "ignore",
        Hash = "ignore"
    )]
    pub span: Span,
}

impl<T> Spanned<T> {
    /// Create a new spanned value
    pub fn new(inner: T, span: Span) -> Self {
        Self { inner, span }
    }

    /// Map the inner value
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Spanned<U> {
        Spanned {
            inner: f(self.inner),
            span: self.span,
        }
    }
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
