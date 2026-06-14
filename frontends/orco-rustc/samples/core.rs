#![feature(sized_hierarchy)]
use std::marker::PointeeSized;

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct NonNull<T: PointeeSized> {
    pointer: *const T,
}

#[derive(Copy, Clone)]
#[repr(align(2))] // To ensure pointers to this struct always have their lowest bit cleared.
pub struct Argument {
    // ty: ArgumentType<'a>,
}

#[derive(Copy, Clone)]
pub struct Arguments {
    template: NonNull<u8>,
    args: NonNull<Argument>,
}

fn main() {}
