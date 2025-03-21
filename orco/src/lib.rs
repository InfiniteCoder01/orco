#![doc = include_str!("../README.md")]
pub mod diagnostic;
pub use diagnostic::DiagCtx;

pub mod backend;

pub trait Module {
    //
}
