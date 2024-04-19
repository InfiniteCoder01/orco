use std::num::NonZeroU16;

/// A named item (for example, if you parse a Function, you'll get Named<Function>, because the
/// function itself doen't store a name)
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Named<T> {
    /// Name
    pub name: String,
    /// Value
    pub value: T,
}

impl<T> Named<T> {
    /// Create a new named item
    pub fn new(name: String, item: T) -> Self {
        Self { name, value: item }
    }

    /// Map the value, preserving the name
    pub fn map<U>(self, mapper: impl Fn(T) -> U) -> Named<U> {
        Named {
            name: self.name,
            value: mapper(self.value),
        }
    }
}

pub(crate) fn numeric_type_size(name: &str, prefix: &str) -> Option<NonZeroU16> {
    name.strip_prefix(prefix)
        .and_then(|bits| bits.parse::<u32>().ok())
        .and_then(|bits| (bits / 8).try_into().ok())
        .and_then(NonZeroU16::new)
}
