use std::{
    marker::{PhantomData, Unsize},
    sync::{Arc, RwLock, Weak},
};

// * Guard
/// Guard to make sure [SymbolBox] can't be dropped when it's accessed via [SymbolRef] (or the other way around)
pub struct Guard<'a, T: ?Sized>(Arc<RwLock<T>>, PhantomData<&'a T>);
impl<T: ?Sized> std::ops::Deref for Guard<'_, T> {
    type Target = RwLock<T>;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

// * SymbolBox
/// Smart pointer for your symbols, so they can be referenced using [SymbolRef]
pub struct SymbolBox<T, H: ?Sized> {
    object: Arc<RwLock<T>>,
    references: Vec<Weak<RwLock<H>>>,
}

impl<T, H: ?Sized> SymbolBox<T, H> {
    /// Create a new [SymbolBox] from it's contents
    pub fn new(object: T) -> Self {
        Self {
            object: Arc::new(RwLock::new(object)),
            references: Vec::new(),
        }
    }

    /// Access contents of this [SymbolBox]
    #[inline]
    pub fn object(&self) -> &RwLock<T> {
        &self.object
    }

    /// Get a list of references to this [SymbolBox] and return their [SymbolRefHandler]s
    pub fn references(&mut self) -> Vec<Guard<H>> {
        let mut references = Vec::with_capacity(self.references.len());
        self.references.retain(|reference| {
            if let Some(reference) = reference.upgrade() {
                references.push(Guard(reference, PhantomData));
                true
            } else {
                false
            }
        });

        references
    }

    /// Create a new [SymbolRef] referencing this [SymbolBox]
    #[inline]
    pub fn new_ref<RT: ?Sized, RH>(&mut self, handler: RH) -> SymbolRef<RT, RH>
    where
        T: Unsize<RT>,
        RH: Unsize<H>,
    {
        let mut symbol_ref = SymbolRef::new(handler);
        symbol_ref.bind(self);
        symbol_ref
    }
}

/// Reference to [SymbolBox], invalidates, if SymbolBox drops
pub struct SymbolRef<T: ?Sized, H: ?Sized> {
    object: Option<Weak<RwLock<T>>>,
    handler: Arc<RwLock<H>>,
}

impl<T: ?Sized, H: ?Sized> SymbolRef<T, H> {
    /// Create a new SymbolRef from [SymbolBoxAccess]
    pub fn new(handler: H) -> Self
    where
        H: Sized,
    {
        Self {
            object: None,
            handler: Arc::new(RwLock::new(handler)),
        }
    }

    /// Bind this [SymbolRef] to a [SymbolBox]
    pub fn bind<BT, BH: ?Sized>(&mut self, symbol_box: &mut SymbolBox<BT, BH>)
    where
        BT: Unsize<T>,
        H: Unsize<BH>,
    {
        let handler: Arc<RwLock<BH>> = self.handler.clone();
        symbol_box.references.push(Arc::downgrade(&handler));

        let object: Arc<RwLock<T>> = symbol_box.object.clone();
        self.object = Some(Arc::downgrade(&object));
    }

    /// Access contents of the [SymbolBox]
    pub fn object(&self) -> Option<Guard<T>> {
        self.object
            .as_ref()
            .and_then(|object| object.upgrade().map(|object| Guard(object, PhantomData)))
    }

    /// Get the [SymbolRefHandler] associated with this [SymbolRef]
    pub fn handler(&self) -> &RwLock<H> {
        &self.handler
    }

    /// Cast underlying handler type
    pub fn cast<NH: ?Sized>(self) -> SymbolRef<T, NH>
    where
        H: Unsize<NH>,
    {
        SymbolRef {
            object: self.object,
            handler: self.handler,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert2::*;

    struct A;
    struct B;
    trait TA {}
    trait TB {}

    impl TA for A {}
    impl TB for B {}

    #[test]
    fn test_box() {
        let mut symbol_box = SymbolBox::<_, dyn TB>::new(A);
        check!(symbol_box.references().len() == 0);

        let symbol_ref = symbol_box.new_ref::<dyn TA, _>(B);
        check!(symbol_box.references().len() == 1);
        check!(symbol_ref.object().is_some());

        drop(symbol_box);
        check!(symbol_ref.object().is_none());
    }

    #[test]
    fn test_ref() {
        let mut symbol_box = SymbolBox::<_, dyn TB>::new(A);
        check!(symbol_box.references().len() == 0);

        let symbol_ref = symbol_box.new_ref::<dyn TA, _>(B);
        check!(symbol_box.references().len() == 1);
        check!(symbol_ref.object().is_some());

        drop(symbol_ref);
        check!(symbol_box.references().len() == 0);
    }

    #[test]
    fn test_safety_drop_ref() {
        let mut symbol_box = SymbolBox::<_, dyn TB>::new(A);
        check!(symbol_box.references().len() == 0);
        check!(Arc::strong_count(&symbol_box.object) == 1);
        check!(Arc::weak_count(&symbol_box.object) == 0);

        let symbol_ref = symbol_box.new_ref::<dyn TA, _>(B);
        check!(symbol_box.references().len() == 1);
        check!(Arc::strong_count(&symbol_box.object) == 1);
        check!(Arc::weak_count(&symbol_box.object) == 1);

        drop(symbol_ref);
        check!(symbol_box.references().len() == 0);
        check!(Arc::strong_count(&symbol_box.object) == 1);
        check!(Arc::weak_count(&symbol_box.object) == 0);
    }

    #[test]
    fn test_safety_drop_box() {
        let mut symbol_box = SymbolBox::<_, dyn TB>::new(A);
        let symbol_ref = symbol_box.new_ref::<dyn TA, _>(B);
        check!(Arc::strong_count(&symbol_ref.handler) == 1);
        check!(Arc::weak_count(&symbol_ref.handler) == 1);

        drop(symbol_box);
        check!(Arc::strong_count(&symbol_ref.handler) == 1);
        check!(Arc::weak_count(&symbol_ref.handler) == 0);
        check!(symbol_ref.object().is_none());
    }
}
