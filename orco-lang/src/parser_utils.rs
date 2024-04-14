use std::num::NonZeroU16;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Named<T> {
    pub name: String,
    pub value: T,
}

impl<T> Named<T> {
    pub fn new(name: String, item: T) -> Self {
        Self { name, value: item }
    }

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
