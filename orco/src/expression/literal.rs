/// Literals
#[derive(Clone)]
pub enum Literal {
    /// Unsigned integer literal, holding type and value
    Integer(u128, crate::Type),
}

impl Literal {
    /// Get the type of this literal
    pub fn r#type(&self) -> &crate::Type {
        match self {
            Self::Integer(_, r#type) => r#type,
        }
    }
}

impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Integer(literal, _) => write!(f, "{}", literal),
        }
    }
}
