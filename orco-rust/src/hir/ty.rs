#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
    Unit,
    Path(super::Path),
    Int(u16),
    Unsigned(u16),
}

impl From<&syn::Type> for Type {
    fn from(value: &syn::Type) -> Self {
        match value {
            syn::Type::Array(type_array) => todo!(),
            syn::Type::BareFn(type_bare_fn) => todo!(),
            syn::Type::Group(type_group) => todo!(),
            syn::Type::ImplTrait(type_impl_trait) => todo!(),
            syn::Type::Infer(type_infer) => todo!(),
            syn::Type::Macro(type_macro) => todo!(),
            syn::Type::Never(type_never) => todo!(),
            syn::Type::Paren(paren) => paren.elem.as_ref().into(),
            syn::Type::Path(path) => Type::Path((&path.path).into()),
            syn::Type::Ptr(type_ptr) => todo!(),
            syn::Type::Reference(type_reference) => todo!(),
            syn::Type::Slice(type_slice) => todo!(),
            syn::Type::TraitObject(type_trait_object) => todo!(),
            syn::Type::Tuple(type_tuple) => todo!(),
            syn::Type::Verbatim(token_stream) => todo!(),
            _ => todo!(),
        }
    }
}
