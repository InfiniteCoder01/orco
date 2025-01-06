use crate::{expression::Function, types::FunctionSignature};
type Intrinsic = crate::ArcLock<Function>;

fn make_intrinsic(name: &str, signature: FunctionSignature) -> Intrinsic {
    std::sync::Arc::new(std::sync::RwLock::new(Function::external(
        name.to_owned(),
        signature,
    )))
}

/// See [self]
pub struct Intrinsics {
    /// Return from a function. Only present inside of a function. Might depend on ABI
    pub r#return: Option<Intrinsic>,
}

impl Intrinsics {
    /// Create a new set of intrinsics
    pub fn new() -> Self {
        Self { r#return: None }
    }

    pub(super) fn r#return(&self, signature: &FunctionSignature) -> Intrinsic {
        make_intrinsic(
            "orco::intrinsics::return",
            FunctionSignature::new(vec![(None, signature.return_type)], crate::Type::Never),
        )
    }
}

impl Default for Intrinsics {
    fn default() -> Self {
        Self::new()
    }
}
