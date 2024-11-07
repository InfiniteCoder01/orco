use super::*;

/// Literals
#[derive(Clone, Copy)]
pub enum Literal<'a, M: Mutability = Imm> {
    /// See [IntegerLiteral]
    Integer(M::Ref<'a, dyn IntegerLiteral>),
}

impl<M: Mutability> std::fmt::Display for Literal<'_, M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Integer(literal) => (&**literal).fmt(f),
        }
    }
}

/// Unsigned integer literal
pub trait IntegerLiteral {
    /// Get the value
    fn value(&self) -> u128;
}

impl std::fmt::Display for &dyn IntegerLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}
