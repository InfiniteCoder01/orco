use super::*;

/// Operators are everything that happens.
/// `break`, `return`, adding two numbers together are all operators
pub mod operators;
pub use operators::{AsOperator, Operator};
/// See [Block]
pub mod block;
pub use block::Block;
/// Everything related to variables
pub mod variables;
pub use variables::VariableDeclaration;
/// See [Literal]
pub mod literal;
pub use literal::Literal;

/// Expressions in orco are all the actual code. Statements are expressions
#[derive(MutrefCloneCopy)]
pub enum Expression<'a, M: Mutability = Imm> {
    /// See [Block]
    Block(M::Ref<'a, dyn Block>),
    /// See [operators]
    Operator(M::Ref<'a, dyn AsOperator>),
    /// See [VariableDeclaration]
    VariableDeclaration(M::Ref<'a, dyn VariableDeclaration>),
    /// See [Literal]
    Literal(Literal<'a, M>),
}

impl<M: Mutability> Expression<'_, M> {
    /// Get the type of this expression
    pub fn r#type(&self) -> Type {
        match self {
            Self::Block(_block) => Type::Unit,
            Self::Operator(_op) => todo!(),
            Self::VariableDeclaration(_) => Type::Unit,
            Self::Literal(literal) => literal.r#type(),
        }
    }
}

impl<M: Mutability> std::fmt::Display for Expression<'_, M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Block(block) => (&**block).fmt(f),
            Self::Operator(op) => (&**op).fmt(f),
            Self::VariableDeclaration(decl) => (&**decl).fmt(f),
            Self::Literal(literal) => literal.fmt(f),
        }
    }
}
