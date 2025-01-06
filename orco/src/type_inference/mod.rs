/// Intrinsics in OrCo are basic building blocks, functions
/// that are necessary to make programs, as every single action
/// is a function.
pub mod intrinsics;

/// This struct holds type inference state.
/// On the first pass it's passed to every node
/// as a mutable reference and gets filled with
/// type variables, on the second pass it's passed
/// as an immutable reference and is used to
/// finalise the types and send out diagnostics
pub struct TypeInferenceContext {
    /// See [intrinsics]
    pub intrinsics: intrinsics::Intrinsics,
}

impl TypeInferenceContext {
    /// Create a new blank type inference context
    pub fn new() -> Self {
        Self {
            intrinsics: intrinsics::Intrinsics::new(),
        }
    }

    pub fn enter_function(&mut self) {
        self.intrinsics
            .r#return
            .insert(self.intrinsics.return_default.clone());
    }

    pub fn exit_function(&mut self) {
        self.intrinsics.r#return.take();
    }
}

impl Default for TypeInferenceContext {
    fn default() -> Self {
        Self::new()
    }
}
