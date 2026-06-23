#![feature(sized_hierarchy)]
#![feature(const_trait_impl)]
#![feature(const_destruct)]
use std::marker::{Destruct, PointeeSized};

pub const trait Clone: Sized {
    #[must_use = "cloning is often expensive and is not expected to have side effects"]
    fn clone(&self) -> Self;

    #[inline]
    fn clone_from(&mut self, source: &Self)
    where
        Self: [const] Destruct,
    {
        *self = source.clone()
    }
}

#[repr(transparent)]
pub struct NonNull<T: PointeeSized> {
    pointer: *const T,
}

impl<T> Clone for NonNull<T> {
    fn clone(&self) -> Self {
        Self {
            pointer: self.pointer,
        }
    }
}

#[repr(align(2))] // To ensure pointers to this struct always have their lowest bit cleared.
pub struct Argument {
    // ty: ArgumentType<'a>,
}

impl Clone for Argument {
    fn clone(&self) -> Self {
        Self {}
    }
}

pub struct Arguments {
    template: NonNull<u8>,
    args: NonNull<Argument>,
}

impl Clone for Arguments {
    fn clone(&self) -> Self {
        Self {
            template: self.template.clone(),
            args: self.args.clone(),
        }
    }
}

fn main() {
    let x = NonNull {
        pointer: 0 as *const u8,
    };
    x.clone();
}
