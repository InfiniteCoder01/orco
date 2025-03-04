use super::cl;

pub(crate) struct FunctionDecl {
    pub(crate) id: cl::FuncId,
    pub(crate) name: String,
    pub(crate) signature: cl::Signature,
}

pub struct FunctionBuilder<'a>(pub(crate) cl::FunctionBuilder<'a>);

impl FunctionBuilder<'_> {}
