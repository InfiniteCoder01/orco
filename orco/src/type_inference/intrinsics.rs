use crate::{expression::Function, types::FunctionSignature};
type IntrinsicFunction = crate::ArcLock<Function>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Intrinsic {
    Return,
    Branch,
}

fn make_intrinsic(signature: FunctionSignature, intrinsic: Intrinsic) -> IntrinsicFunction {
    std::sync::Arc::new(std::sync::RwLock::new(Function::intrinsic(
        signature, intrinsic,
    )))
}

/// See [self]
pub struct Intrinsics {
    /// Return from a function. Only present inside of a function. Might depend on ABI
    /// Signature: `fn orco::intrinsics::return(value: return_type) -> !`
    pub r#return: Option<IntrinsicFunction>,
}

impl Intrinsics {
    /// Create a new set of intrinsics
    pub fn new() -> Self {
        Self { r#return: None }
    }

    pub(super) fn r#return(&self, signature: &FunctionSignature) -> IntrinsicFunction {
        make_intrinsic(
            crate::function_signature![(value: {signature.return_type.as_ref().clone()}) -> ! transparent],
            Intrinsic::Return,
        )
    }

    /// Branch, basically just an if
    /// Signature: `fn orco::intrinsics::branch<T>(cond: bool, then: fn () -> T, else: fn () -> T) -> T`
    pub fn branch(&self, r#type: crate::Type) -> IntrinsicFunction {
        make_intrinsic(
            crate::function_signature![(cond: bool, then: (fn () -> {r#type.clone()} transparent), else: (fn () -> {r#type.clone()} transparent)) -> {r#type} transparent],
            Intrinsic::Branch,
        )
    }
}

impl Default for Intrinsics {
    fn default() -> Self {
        Self::new()
    }
}
