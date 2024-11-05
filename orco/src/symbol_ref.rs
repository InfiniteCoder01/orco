use std::{ptr::NonNull, sync::RwLock};

type ReferenceList<T> = RwLock<Vec<NonNull<SymbolRefInner<T>>>>;

fn check_lock_result<Guard>(result: std::sync::TryLockResult<Guard>, what: &str) -> Guard {
    match result {
        Ok(references) => references,
        Err(std::sync::TryLockError::Poisoned(err)) => {
            eprintln!(
                "Something went terribly wrong! {} got poisoned! Error: {}",
                what, err
            );
            eprintln!("Backtrace:\n{}", std::backtrace::Backtrace::capture());
            std::process::exit(1);
        }
        Err(std::sync::TryLockError::WouldBlock) => {
            eprintln!("Something went terribly wrong! {} is locked on the same thread (WouldBlock locking error)!", what);
            eprintln!("Backtrace:\n{}", std::backtrace::Backtrace::capture());
            std::process::exit(1);
        }
    }
}

// * SymbolBox
struct SymbolBoxInner<T> {
    object: RwLock<T>,
    references: ReferenceList<T>,
}

impl<T> SymbolBoxInner<T> {
    pub fn access<'a>(&'a self) -> SymbolBoxAccess<'a, T> {
        SymbolBoxAccess(self.object(), &self.references)
    }

    #[inline]
    pub fn object(&self) -> &RwLock<T> {
        &self.object
    }

    pub fn new_ref(&self) -> SymbolRef<T> {
        SymbolRef::new(self.access())
    }
}

impl<T> std::ops::Drop for SymbolBoxInner<T> {
    fn drop(&mut self) {
        let references = check_lock_result(self.references.try_read(), "SymbolBox reference list");
        let references = match self.references.try_read() {
            Ok(references) => references,
            Err(std::sync::TryLockError::Poisoned(err)) => {
                eprintln!("Something went terribly wrong! SymbolBox reference list got poisoned!!! Terminating the whole thing! Error: {}", err);
                std::process::exit(1);
            }
            Err(std::sync::TryLockError::WouldBlock) => {
                eprintln!("Something went terribly wrong! SymbolBox reference list is locked on the same thread as the destructor got called!!! Terminating the whole thing!");
                std::process::exit(1);
            }
        };
        for reference in references.iter() {
            unsafe { reference.as_ref() }
                .symbol_box
                .try_write()
                .unwrap()
                .take();
        }
    }
}

/// Smart pointer for your symbols, so they can be referenced using [SymbolRef]
pub struct SymbolBox<T>(std::pin::Pin<Box<SymbolBoxInner<T>>>);

impl<T> SymbolBox<T> {
    pub fn new(object: T) -> Self {
        Self(Box::pin(SymbolBoxInner {
            object: RwLock::new(object),
            references: RwLock::default(),
        }))
    }

    pub fn access<'a>(&'a self) -> SymbolBoxAccess<'a, T> {
        self.0.access()
    }

    pub fn object(&self) -> &RwLock<T> {
        self.0.object()
    }

    pub fn new_ref(&self) -> SymbolRef<T> {
        self.0.new_ref()
    }
}

// * SymbolBoxAccess
pub struct SymbolBoxAccess<'a, T: ?Sized>(&'a RwLock<T>, &'a ReferenceList<T>);

impl<'a, T: ?Sized> Clone for SymbolBoxAccess<'a, T> {
    fn clone(&self) -> Self {
        SymbolBoxAccess(self.0, self.1)
    }
}

impl<'a, T: ?Sized> Copy for SymbolBoxAccess<'a, T> {}

impl<'a, T: ?Sized> SymbolBoxAccess<'a, T> {
    pub fn object(&self) -> &RwLock<T> {
        self.0
    }

    pub fn new_ref(&self) -> SymbolRef<T> {
        SymbolRef::new(*self)
    }
}

// * SymbolRef
struct SymbolRefInner<T: ?Sized> {
    symbol_box: RwLock<Option<(NonNull<RwLock<T>>, NonNull<ReferenceList<T>>)>>,
}

impl<T: ?Sized> Drop for SymbolRefInner<T> {
    fn drop(&mut self) {
        if let Ok(Some((_, references))) = self.symbol_box.try_read().map(|symbox| *symbox) {
            let mut references = match unsafe { references.as_ref() }.try_write() {
                Ok(references) => references,
                Err(std::sync::TryLockError::Poisoned(err)) => {
                    eprintln!("Something went terribly wrong! SymbolBox reference list got poisoned!!! Terminating the whole thing! Error: {}", err);
                    std::process::exit(1);
                }
                Err(std::sync::TryLockError::WouldBlock) => {
                    eprintln!("Something went terribly wrong! SymbolBox reference list is locked on the same thread as the destructor of SymbolRef got called!!! Terminating the whole thing!");
                    std::process::exit(1);
                }
            };
            references.retain(|reference| {
                reference.as_ptr() as *const SymbolRefInner<T> != self as *const SymbolRefInner<T>
            });
        }
    }
}

pub struct SymbolRef<T: ?Sized>(std::pin::Pin<Box<SymbolRefInner<T>>>);

impl<T: ?Sized> SymbolRef<T> {
    pub fn new(access: SymbolBoxAccess<T>) -> Self {
        let inner = Box::pin(SymbolRefInner {
            symbol_box: RwLock::new(Some((access.0.into(), access.1.into()))),
        });
        access.1.try_write().unwrap().push((&*inner).into());
        Self(inner)
    }
}

#[test]
fn test() {
    use assert2::*;
    let symbol_box = SymbolBox::new(42);
    *symbol_box.object().try_write().unwrap() = 69;
    check!(*symbol_box.object().try_read().unwrap() == 69);

    // let symbol_ref = symbol_box.new_ref();
    // *symbol_box.get_mut() += 1;
    // check!(*symbol_box.get() == 70);
    // check!(*symbol_ref.get() == 70);
    // symbol_ref.
}

#[test]
fn test_safety_drop_ref() {
    use assert2::*;
    let symbol_box = SymbolBox::new(42);
    check!(symbol_box.0.references.try_read().unwrap().len() == 0);
    let symbol_ref = symbol_box.new_ref();
    check!(symbol_box.0.references.try_read().unwrap().len() == 1);
    drop(symbol_ref);
    check!(symbol_box.0.references.try_read().unwrap().len() == 0);
}

#[test]
fn test_safety_drop_box() {
    use assert2::*;
    let symbol_box = SymbolBox::new(42);
    let symbol_ref = symbol_box.new_ref();
    drop(symbol_box);
    check!(symbol_ref.0.symbol_box.try_read().unwrap().is_none());
}

#[test]
#[should_panic]
fn test_panicking() {
    use assert2::*;
    let symbol_box = SymbolBox::new(42);
    let symbol_ref = symbol_box.new_ref();
    std::thread::spawn(move || {
        let references = symbol_ref.0.symbol_box.try_read().unwrap().unwrap().1;
        let references = unsafe { references.as_ref() };
        let references = references.try_write().unwrap();
        panic!();
    });
}
