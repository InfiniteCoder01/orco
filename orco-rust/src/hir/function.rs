use super::{BodyId, Path, Type};
use crate::Context;

#[derive(Clone, Debug)]
pub struct Signature {
    pub parameters: Vec<Type>,
    pub return_type: Type,
}

impl Signature {
    pub fn parse(ctx: &mut Context, value: syn::Signature) -> Self {
        Self {
            parameters: value
                .inputs
                .iter()
                .map(|arg| match arg {
                    syn::FnArg::Receiver(receiver) => todo!(),
                    syn::FnArg::Typed(arg) => Type::parse(ctx, &arg.ty),
                })
                .collect(),
            return_type: match value.output {
                syn::ReturnType::Default => Type::unit(),
                syn::ReturnType::Type(_, ty) => Type::parse(ctx, &ty),
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct Function {
    pub path: Path,
    pub signature: Signature,
    pub body: BodyId,
}
