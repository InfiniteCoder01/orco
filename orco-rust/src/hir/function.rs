use super::{BodyId, Path, Type};

#[derive(Clone, Debug)]
pub struct Signature {
    pub parameters: Vec<Type>,
    pub return_type: Type,
}

impl Signature {
    pub fn parse(signature: syn::Signature) -> Self {
        Self {
            parameters: signature
                .inputs
                .iter()
                .map(|arg| match arg {
                    syn::FnArg::Receiver(_receiver) => todo!(),
                    syn::FnArg::Typed(arg) => Type::parse(&arg.ty),
                })
                .collect(),
            return_type: match signature.output {
                syn::ReturnType::Default => Type::unit(),
                syn::ReturnType::Type(_, ty) => Type::parse(&ty),
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
