#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
    Tuple(Vec<Type>),
    Path(super::Path),
    Int(u16),
    Unsigned(u16),
}

impl Type {
    pub fn parse(ty: &syn::Type) -> Self {
        match ty {
            syn::Type::Array(_type_array) => todo!(),
            syn::Type::BareFn(_type_bare_fn) => todo!(),
            syn::Type::Group(_type_group) => todo!(),
            syn::Type::ImplTrait(_type_impl_trait) => todo!(),
            syn::Type::Infer(_type_infer) => todo!(),
            syn::Type::Macro(_type_macro) => todo!(),
            syn::Type::Never(_type_never) => todo!(),
            syn::Type::Paren(paren) => Self::parse(paren.elem.as_ref()),
            syn::Type::Path(path) => Self::Path((&path.path).into()),
            syn::Type::Ptr(_type_ptr) => todo!(),
            syn::Type::Reference(_type_reference) => todo!(),
            syn::Type::Slice(_type_slice) => todo!(),
            syn::Type::TraitObject(_type_trait_object) => todo!(),
            syn::Type::Tuple(tuple) => Self::Tuple(tuple.elems.iter().map(Type::parse).collect()),
            syn::Type::Verbatim(_token_stream) => todo!(),
            _ => todo!(),
        }
    }

    pub fn unit() -> Type {
        Type::Tuple(vec![])
    }
}
