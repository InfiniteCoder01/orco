use interned_string::Intern;
use orco::diagnostic::Span;
pub type Symbol = interned_string::IString;

#[derive(Clone)]
pub struct Ident {
    pub name: Symbol,
    pub span: Span,
}

impl std::fmt::Debug for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Ident")
            .field(&self.name)
            .field(&self.span)
            .finish()
    }
}

impl std::fmt::Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl PartialEq for Ident {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}

impl Eq for Ident {}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd for Ident {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl Ord for Ident {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl std::hash::Hash for Ident {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}

impl From<&syn::Ident> for Ident {
    fn from(value: &syn::Ident) -> Self {
        Self {
            name: value.to_string().intern(),
            span: value.span().byte_range().into(),
        }
    }
}

impl From<&str> for Ident {
    fn from(value: &str) -> Self {
        Self {
            name: value.intern(),
            span: (0..value.len()).into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Path(pub Vec<Ident>);

impl Path {
    pub fn empty() -> Self {
        Self(Vec::new())
    }

    pub fn single(segment: impl Into<Ident>) -> Self {
        Self(vec![segment.into()])
    }

    pub fn push(&mut self, segment: impl Into<Ident>) {
        self.0.push(segment.into());
    }

    pub fn pop(&mut self) {
        self.0.pop();
    }

    pub fn join(mut self, segment: impl Into<Ident>) -> Self {
        self.push(segment);
        self
    }

    pub fn parse(path: &str) -> Self {
        Self(path.split("::").map(|segment| segment.into()).collect())
    }

    pub fn span(&self) -> Span {
        let first = self.0.first().unwrap().span;
        let last = self.0.last().unwrap().span;
        Span::from(first.offset()..last.offset() + last.len())
    }
}

impl From<&syn::Path> for Path {
    fn from(value: &syn::Path) -> Self {
        Self(
            value
                .segments
                .iter()
                .map(|segment| (&segment.ident).into())
                .collect(),
        )
    }
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            return write!(f, "<Empty Path>");
        }
        for (idx, segment) in self.0.iter().enumerate() {
            if idx > 0 {
                write!(f, "::")?;
            }
            segment.fmt(f)?;
        }
        Ok(())
    }
}
