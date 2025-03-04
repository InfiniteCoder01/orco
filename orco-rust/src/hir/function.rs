use super::{BodyHandle, Path, Type};

#[derive(Clone, Debug)]
pub struct Signature {
    pub parameters: Vec<Type>,
    pub return_type: Type,
}

impl From<syn::Signature> for Signature {
    fn from(value: syn::Signature) -> Self {
        Self {
            parameters: value
                .inputs
                .iter()
                .map(|arg| match arg {
                    syn::FnArg::Receiver(receiver) => todo!(),
                    syn::FnArg::Typed(arg) => arg.ty.as_ref().into(),
                })
                .collect(),
            return_type: match value.output {
                syn::ReturnType::Default => Type::Unit,
                syn::ReturnType::Type(_, ty) => ty.as_ref().into(),
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct Function {
    pub path: Path,
    pub signature: Signature,
    pub body: BodyHandle,
}
