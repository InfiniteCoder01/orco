use super::*;

/// See [Block]
pub mod block;
pub use block::Block;
/// Constructs that affect control flow, like [Return], [Break], etc. Conditionals not included
pub mod control_flow;
pub use control_flow::Return;
/// See [Literal]
pub mod literal;
pub use literal::Literal;

/// Expressions in orco are all the actual code. Statements are expressions
#[derive(MutrefCloneCopy)]
pub enum Expression<'a, M: Mutability = Imm> {
    /// See [Block]
    Block(M::Ref<'a, dyn Block>),
    /// See [Return]
    Return(M::Ref<'a, dyn Return>),
    /// See [Literal]
    Literal(Literal<'a, M>),
}

impl<M: Mutability> std::fmt::Display for Expression<'_, M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Block(block) => (&**block).fmt(f),
            Self::Return(expression) => (&**expression).fmt(f),
            Self::Literal(literal) => literal.fmt(f),
        }
    }
}
