use super::*;

/// C types
#[derive(Clone, PartialEq, Eq, Parse, ToTokens)]
pub enum Type {
    Void(kw::Void),
    Int(kw::Int),
}

impl Type {
    pub fn as_orco(&self) -> orco::Type {
        match self {
            Type::Void(_) => orco::Type::Unit,
            Type::Int(_) => orco::Type::Integer(32),
        }
    }
}
