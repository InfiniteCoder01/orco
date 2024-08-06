pub use assert2::*;
pub use orco::ir;

#[macro_export]
macro_rules! make_type_inference {
    ($type_inference: ident, $errors: ident) => {
        use orco::type_inference::TypeInference;
        let mut $errors = Vec::new();
        let root = ir::Module::default();
        let mut $type_inference =
            TypeInference::new(&mut $errors, orco::Interpreter::default(), &root);
    };
}
