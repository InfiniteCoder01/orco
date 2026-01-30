pub trait SimpleIterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
    fn peekable(self) -> Peekable<Self, Self::Item>
    where
        Self: Sized,
    {
        Peekable {
            iter: self,
            peeked: None,
        }
    }
}

pub struct Peekable<T, I> {
    iter: T,
    peeked: Option<I>,
}

impl<I: Clone, T: SimpleIterator<Item = I>> Peekable<T, I> {
    pub fn peek(&mut self) -> Option<I> {
        if self.peeked.is_none() {
            self.peeked = self.iter.next();
        }
        self.peeked.clone()
    }
}

impl<I, T: SimpleIterator<Item = I>> SimpleIterator for Peekable<T, I> {
    type Item = T::Item;
    fn next(&mut self) -> Option<T::Item> {
        match &self.peeked {
            Some(_) => std::mem::replace(&mut self.peeked, None),
            None => self.iter.next(),
        }
    }
}

pub struct Iota(usize);

impl Iota {
    pub fn new() -> Self {
        Iota(0)
    }
}

impl SimpleIterator for Iota {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        let prev = self.0;
        self.0 = prev + 1;
        Some(prev)
    }
}

fn main() {
    let mut iota = Iota::new();
    assert_eq!(iota.next(), Some(0));
    assert_eq!(iota.next(), Some(1));
    assert_eq!(iota.next(), Some(2));
    assert_eq!(iota.next(), Some(3));

    let mut peekable_iota = iota.peekable();
    assert_eq!(peekable_iota.peek(), Some(4));
    assert_eq!(peekable_iota.peek(), Some(4));
    assert_eq!(peekable_iota.peek(), Some(4));
    assert_eq!(peekable_iota.next(), Some(4));
    assert_eq!(peekable_iota.next(), Some(5));
    assert_eq!(peekable_iota.next(), Some(6));
}
