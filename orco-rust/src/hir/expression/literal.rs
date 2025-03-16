use crate::Context;
use crate::backend::cl::InstBuilder;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Literal {
    Int(u128),
}

impl Literal {
    pub fn parse(_ctx: &mut Context, literal: &syn::Lit) -> Self {
        match literal {
            syn::Lit::Str(_) => todo!(),
            syn::Lit::ByteStr(_) => todo!(),
            syn::Lit::CStr(_) => todo!(),
            syn::Lit::Byte(_) => todo!(),
            syn::Lit::Char(_) => todo!(),
            syn::Lit::Int(value) => {
                Self::Int(value.base10_digits().parse().expect("Invalid int literal"))
            }
            syn::Lit::Float(_) => todo!(),
            syn::Lit::Bool(_) => todo!(),
            syn::Lit::Verbatim(_) => todo!(),
            _ => todo!(),
        }
    }
}

impl Literal {
    pub fn build(
        &self,
        builder: &mut crate::backend::FunctionBuilder,
    ) -> crate::backend::cl::Value {
        match self {
            Literal::Int(value) => builder
                .0
                .ins()
                .iconst(crate::backend::cl::types::I32, *value as i64),
        }
    }
}
