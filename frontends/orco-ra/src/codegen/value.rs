/// Wrapper around a value, providing Unit and Never variants
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AType<V> {
    Value(V),
    Unit,
    Never,
}

impl<V> AType<V> {
    pub fn unwrap(self) -> V {
        match self {
            Self::Value(value) => value,
            Self::Unit => panic!("called 'unwrap' on a unit AType"),
            Self::Never => panic!("called 'unwrap' on a never AType"),
        }
    }

    pub fn map<U>(self, f: impl FnOnce(V) -> U) -> AType<U> {
        match self {
            Self::Value(v) => AType::Value(f(v)),
            Self::Unit => AType::Unit,
            Self::Never => AType::Never,
        }
    }
}
