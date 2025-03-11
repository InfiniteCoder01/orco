use crate::backend::cl::InstBuilder;

#[derive(Clone, Debug)]
pub struct Literal {
    pub lit: syn::Lit,
}

impl From<syn::Lit> for Literal {
    fn from(value: syn::Lit) -> Self {
        Self { lit: value }
    }
}

impl Literal {
    pub fn build(
        &self,
        builder: &mut crate::backend::FunctionBuilder,
    ) -> crate::backend::cl::Value {
        match &self.lit {
            syn::Lit::Str(lit_str) => todo!(),
            syn::Lit::ByteStr(lit_byte_str) => todo!(),
            syn::Lit::CStr(lit_cstr) => todo!(),
            syn::Lit::Byte(lit_byte) => todo!(),
            syn::Lit::Char(lit_char) => todo!(),
            syn::Lit::Int(int) => builder.0.ins().iconst(
                crate::backend::cl::types::I32,
                int.base10_parse::<i64>().unwrap(),
            ),
            syn::Lit::Float(lit_float) => todo!(),
            syn::Lit::Bool(lit_bool) => todo!(),
            syn::Lit::Verbatim(literal) => todo!(),
            _ => todo!(),
        }
    }
}
