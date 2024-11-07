use super::*;

/// C types
#[derive(Clone, PartialEq, Eq, Parse, ToTokens)]
pub enum Type {
    Void(kw::Void),
    Int(kw::Int),
}
