pub use crate::FunctionId;

/// Function-related structures
pub mod function;
pub use function::{FunctionBuilder, SSAValue, SignatureBuilder};

/// Compilation backend
pub trait Backend {
    /// Decalre a function. All functions must be declared before any function can be built.
    /// See [SignatureBuilder]
    fn declare_function(
        &mut self,
        id: FunctionId,
        name: impl ToString,
    ) -> Box<dyn SignatureBuilder + '_>;

    /// Build function. See [FunctionBuilder]
    fn build_function(&self, id: FunctionId) -> Box<dyn FunctionBuilder + '_>;
}
