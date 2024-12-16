use parsel::{Parse, ToTokens};

/// Wrapper around [`orco::SymbolBox`] with parsing traits
pub struct SymbolBox<T, H: ?Sized>(orco::SymbolBox<T, H>);

impl<T, H: ?Sized> std::ops::Deref for SymbolBox<T, H> {
    type Target = orco::SymbolBox<T, H>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, H: ?Sized> std::ops::DerefMut for SymbolBox<T, H> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Parse, H: ?Sized> Parse for SymbolBox<T, H> {
    fn parse(input: parsel::syn::parse::ParseStream) -> parsel::Result<Self> {
        Ok(Self(orco::SymbolBox::new(T::parse(input)?)))
    }
}

impl<T: ToTokens, H: ?Sized> ToTokens for SymbolBox<T, H> {
    fn to_tokens(&self, tokens: &mut parsel::TokenStream) {
        self.object().try_read().unwrap().to_tokens(tokens)
    }
}

impl<T: PartialEq, H: ?Sized> PartialEq for SymbolBox<T, H> {
    fn eq(&self, other: &Self) -> bool {
        self.object()
            .try_read()
            .unwrap()
            .eq(&*other.object().try_read().unwrap())
    }
}

impl<T: Eq, H: ?Sized> Eq for SymbolBox<T, H> {}

/// Wrapper around [`orco::SymbolRef`] with parsing traits
pub struct SymbolRef<T: ?Sized, H: ?Sized>(orco::SymbolRef<T, H>);

impl<T: ?Sized, H: Parse> Parse for SymbolRef<T, H> {
    fn parse(input: parsel::syn::parse::ParseStream) -> parsel::Result<Self> {
        Ok(Self(orco::SymbolRef::new(H::parse(input)?)))
    }
}

impl<T: ?Sized, H: ToTokens + ?Sized> ToTokens for SymbolRef<T, H> {
    fn to_tokens(&self, tokens: &mut parsel::TokenStream) {
        self.object().try_read().unwrap().to_tokens(tokens)
    }
}
