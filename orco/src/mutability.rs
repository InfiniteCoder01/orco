/// Trait that represents mutability
pub trait Mutability {
    /// A simple reference
    type Ref<'a, T: ?Sized + 'a>: std::ops::Deref<Target = T>;
}

/// Immutable version
#[derive(Clone, Copy)]
pub struct Imm;

impl Mutability for Imm {
    type Ref<'a, T: ?Sized + 'a> = &'a T;
}

/// Mutable version
pub struct Mut;

impl Mutability for Mut {
    type Ref<'a, T: ?Sized + 'a> = &'a mut T;
}
