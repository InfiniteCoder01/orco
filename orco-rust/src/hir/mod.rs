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

pub struct Context {
    pub diag: crate::DiagCtx,
}

pub fn parse_file(
    ctx: &mut Context,
    hir: &mut Hir,
    filepath: impl AsRef<std::path::Path>,
    mut modpath: Path,
) {
    let filepath = filepath.as_ref();
    let source = std::fs::read_to_string(filepath).unwrap();
    ctx.diag
        .set_source(miette::NamedSource::new(filepath.to_string_lossy(), source.clone()).into());
    let file = match syn::parse_str::<syn::File>(&source) {
        Ok(file) => file,
        Err(err) => {
            ctx.diag.syntax_error(err);
            return;
        }
    };

    for item in file.items {
        match item {
            syn::Item::Fn(r#fn) => {
                modpath.push(&r#fn.sig.ident);
                let path = if r#fn.sig.ident == "main" {
                    Path::single(&r#fn.sig.ident)
                } else {
                    modpath.clone()
                };

                let signature = Signature::parse(r#fn.sig);
                let body = Body::new(Block::parse(&r#fn.block, &path).into());
                let body = hir.bodies.push_get_id(body.into());
                hir.functions.push_get_id(
                    Function {
                        path,
                        signature,
                        body,
                    }
                    .into(),
                );

                modpath.pop();
            }
            _ => todo!(),
        }
    }
}
