pub mod function;
pub use function::{FunctionBuilder, SSAValue, SignatureBuilder};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctionId(pub usize);

impl<T: Into<usize>> From<T> for FunctionId {
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

pub trait Backend {
    fn declare_function(
        &mut self,
        id: FunctionId,
        name: impl ToString,
    ) -> Box<dyn SignatureBuilder + '_>;
    fn build_function(&self, id: FunctionId) -> Box<dyn FunctionBuilder + '_>;
}
