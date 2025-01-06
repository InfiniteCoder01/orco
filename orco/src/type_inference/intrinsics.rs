use crate::expression::Function;
type Intrinsic = crate::ArcLock<Function>;

fn make_intrinsic(name: &str) -> Intrinsic {
    std::sync::Arc::new(std::sync::RwLock::new(Function::external(name.to_owned())))
}

/// See [self]
pub struct Intrinsics {
    pub(super) return_default: Intrinsic,
    /// Return from a function. Only present inside of a function. Might depend on ABI
    pub r#return: Option<Intrinsic>,
}

impl Intrinsics {
    /// Create a new set of intrinsics
    pub fn new() -> Self {
        Self {
            return_default: make_intrinsic("orco::intrinsics::return"),
            r#return: None,
        }
    }
}

impl Default for Intrinsics {
    fn default() -> Self {
        Self::new()
    }
}
