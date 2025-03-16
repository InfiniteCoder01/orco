use crate::Context;
use indexland::{Idx, IndexVec};
use rayon::prelude::*;
use std::sync::RwLock;

pub mod interner;
pub use interner::{Ident, Path, Symbol};

pub mod ty;
pub use ty::Type;

pub mod function;
pub use function::{Function, Signature};

pub mod expression;
pub use expression::{Block, Body, Expression, Literal};

#[derive(Idx)]
pub struct FunctionId(usize);

#[derive(Idx)]
pub struct BodyId(usize);

#[derive(Debug, Default)]
pub struct Hir {
    pub functions: IndexVec<FunctionId, RwLock<Function>>,
    pub bodies: IndexVec<BodyId, RwLock<Body>>,
}

impl Hir {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn resolve(&self, ctx: &Context) {
        self.bodies.as_slice().into_par_iter().for_each(|body| {
            body.write().unwrap().resolve(ctx);
        });
    }
}
