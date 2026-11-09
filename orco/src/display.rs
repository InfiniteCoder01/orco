use crate::Module;

/// Wrapper to display different structs.
pub struct Display<'a, T: ?Sized> {
    #[allow(missing_docs)]
    pub module: &'a Module,
    value: &'a T,
}

impl<'a, T: ?Sized> Copy for Display<'a, T> {}
impl<T: ?Sized> Clone for Display<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> std::ops::Deref for Display<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a, T: ?Sized> Display<'a, T> {
    /// Wrap a new value in a [`Display`] with the same module.
    pub fn project<'b, R: ?Sized>(self, value: &'b R) -> Display<'b, R>
    where
        'a: 'b,
    {
        Display {
            module: self.module,
            value,
        }
    }
}

impl Module {
    /// Wrap a thing in [`Display`].
    pub fn display<'a, T: ?Sized>(&'a self, value: &'a T) -> Display<'a, T> {
        Display {
            module: self,
            value,
        }
    }
}
