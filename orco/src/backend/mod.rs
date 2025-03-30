pub use crate::FunctionId;

pub mod function;
pub use function::{FunctionBuilder, SSAValue, SignatureBuilder};

pub trait Backend {
    fn declare_function(
        &mut self,
        id: FunctionId,
        name: impl ToString,
    ) -> Box<dyn SignatureBuilder + '_>;
    fn build_function(&self, id: FunctionId) -> Box<dyn FunctionBuilder + '_>;
}
