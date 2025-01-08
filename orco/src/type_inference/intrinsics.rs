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
    /// Signature: `fn orco::intrinsics::return(value: return_type) -> !`
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
            crate::function_signature![(value: {signature.return_type.as_ref().clone()}) -> !],
        )
    }

    /// Branch, basically just an if
    /// Signature: `fn orco::intrinsics::branch<T>(cond: bool, then: fn () -> T, else: fn () -> T) -> T`
    pub fn branch(&self, r#type: crate::Type) -> Intrinsic {
        make_intrinsic(
            "orco::intrinsics::branch",
            crate::function_signature![(cond: bool, then: (fn () -> {r#type.clone()}), else: (fn () -> {r#type.clone()})) -> {r#type}],
        )
    }
}

impl Default for Intrinsics {
    fn default() -> Self {
        Self::new()
    }
}
