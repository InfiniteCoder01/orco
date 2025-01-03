#[repr(transparent)]
pub struct Unparse<T>(T);

impl<T> std::ops::Deref for Unparse<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Unparse<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Default> parsel::Parse for Unparse<T> {
    fn parse(_input: parsel::syn::parse::ParseStream) -> parsel::Result<Self> {
        Ok(Self(T::default()))
    }
}

impl<T> parsel::ToTokens for Unparse<T> {
    fn to_tokens(&self, _tokens: &mut parsel::TokenStream) {}
}
