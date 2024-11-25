use parsel::{Parse, ToTokens};

/// Wrapper around [`orco::SymbolBox`] with parsing traits
pub struct SymbolBox<T>(orco::SymbolBox<T>);

impl<T> std::ops::Deref for SymbolBox<T> {
    type Target = orco::SymbolBox<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for SymbolBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Parse> Parse for SymbolBox<T> {
    fn parse(input: parsel::syn::parse::ParseStream) -> parsel::Result<Self> {
        Ok(Self(orco::SymbolBox::new(T::parse(input)?)))
    }
}

impl<T: ToTokens> ToTokens for SymbolBox<T> {
    fn to_tokens(&self, tokens: &mut parsel::TokenStream) {
        self.object().try_read().unwrap().to_tokens(tokens)
    }
}

impl<T: PartialEq> PartialEq for SymbolBox<T> {
    fn eq(&self, other: &Self) -> bool {
        self.object()
            .try_read()
            .unwrap()
            .eq(&*other.object().try_read().unwrap())
    }
}

impl<T: Eq> Eq for SymbolBox<T> {}

/// Wrapper around [`orco::SymbolRef`] with parsing traits
pub struct SymbolRef<I, T: ?Sized> {
    pub ident: I,
    pub reference: Option<orco::SymbolRef<T>>,
}

impl<I: Parse, T: ?Sized> Parse for SymbolRef<I, T> {
    fn parse(input: parsel::syn::parse::ParseStream) -> parsel::Result<Self> {
        Ok(Self::Unresolved(orco::SymbolBox::new(T::parse(input)?)))
    }
}

impl<T: ToTokens> ToTokens for SymbolBox<T> {
    fn to_tokens(&self, tokens: &mut parsel::TokenStream) {
        self.object().try_read().unwrap().to_tokens(tokens)
    }
}

impl<T: PartialEq> PartialEq for SymbolBox<T> {
    fn eq(&self, other: &Self) -> bool {
        self.object()
            .try_read()
            .unwrap()
            .eq(&*other.object().try_read().unwrap())
    }
}

impl<T: Eq> Eq for SymbolBox<T> {}
