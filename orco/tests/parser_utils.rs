pub use assert2::*;
pub use orco::ir;

use orco::diagnostics::Spanned;
use orco::{Span, Src};

pub fn dummy_src() -> Src {
    use std::sync::OnceLock;
    static DUMMY_SRC: OnceLock<Src> = OnceLock::new();
    DUMMY_SRC
        .get_or_init(|| Src::new("dummy".to_owned(), "dummy".into()))
        .clone()
}

pub fn dummy_span() -> Span {
    orco::Span(dummy_src(), 0..5)
}

pub fn dummy_spanned<T>(inner: T) -> Spanned<T> {
    Spanned {
        inner,
        span: dummy_span(),
    }
}

#[macro_export]
macro_rules! make_type_inference {
    ($type_inference: ident, $errors: ident) => {
        use orco::type_inference::TypeInference;
        let return_type = dummy_spanned(ir::Type::Unit);
        let mut $errors = Vec::new();
        let root = ir::Module::default();
        let path = orco::Path::new();
        let mut $type_inference =
            TypeInference::new(&return_type, &mut $errors, &root, &root, &path);
    };
}
