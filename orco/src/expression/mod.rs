use super::*;

/// See [Block]
pub mod block;
pub use block::Block;
/// Constructs that affect control flow, like [Return], [Break], etc. Conditionals not included
pub mod control_flow;
pub use control_flow::Return;
/// Everything related to variables
pub mod variables;
pub use variables::VariableDeclaration;
/// Functions
pub mod functions;
pub use functions::FunctionCall;
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
    /// See [VariableDeclaration]
    VariableDeclaration(M::Ref<'a, dyn VariableDeclaration>),
    /// See [FunctionCall]
    FunctionCall(M::Ref<'a, dyn FunctionCall>),
    /// See [Literal]
    Literal(Literal<'a, M>),
}

impl<M: Mutability> Expression<'_, M> {
    /// Get the type of this expression
    pub fn r#type(&self) -> Type {
        match self {
            Self::Block(_) => todo!(),
            Self::Return(_) => Type::Never,
            Self::VariableDeclaration(_) => Type::Unit,
            Self::FunctionCall(call) => call
                .callee()
                .object()
                .expect("Invalid function reference in the codebase")
                .try_read()
                .unwrap()
                .signature()
                .return_type(),
            Self::Literal(literal) => literal.r#type(),
        }
    }
}

impl<M: Mutability> std::fmt::Display for Expression<'_, M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Block(block) => (&**block).fmt(f),
            Self::Return(expression) => (&**expression).fmt(f),
            Self::VariableDeclaration(decl) => (&**decl).fmt(f),
            Self::FunctionCall(call) => (&**call).fmt(f),
            Self::Literal(literal) => literal.fmt(f),
        }
    }
}
