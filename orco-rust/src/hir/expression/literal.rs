#[derive(Clone, Debug)]
pub struct Literal {
    pub lit: syn::Lit,
}

impl From<syn::Lit> for Literal {
    fn from(value: syn::Lit) -> Self {
        Self { lit: value }
    }
}
