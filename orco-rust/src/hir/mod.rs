use handy::typed::{TypedHandle, TypedHandleMap};

pub mod interner;
pub use interner::{Ident, Path, Symbol};

pub mod ty;
pub use ty::Type;

pub mod function;
pub use function::{Function, Signature};

pub mod expression;
pub use expression::{Block, Body, Expression, Literal};

pub type FunctionHandle = TypedHandle<Function>;
pub type BodyHandle = TypedHandle<Body>;

#[derive(Clone, Debug, Default)]
pub struct Hir {
    pub functions: TypedHandleMap<Function>,
    pub bodies: TypedHandleMap<Body>,
}

impl Hir {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn resolve(&mut self) {
        for function in self.functions.iter_mut() {
            let Type::Path(path) = &function.signature.return_type else {
                continue;
            };
            if path == &Path::parse("i32") {
                function.signature.return_type = Type::Int(32);
            }
        }
    }
}
