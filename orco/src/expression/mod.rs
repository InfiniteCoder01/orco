/// See [Literal]
pub mod literal;
pub use literal::Literal;
/// Function
pub mod function;
pub use function::Function;
/// See [Call]
pub mod call;
pub use call::Call;

/// Expressions in orco are all the actual code. Statements are expressions
pub enum Expression {
    /// See [Literal]
    Literal(Literal),
    /// See [Function]
    Variable(crate::ArcLock<crate::Variable>),
    /// See [Function]
    Function(Function),
    /// See [Call]
    Call(Call),
    /// Invalid expression
    Error,
}

impl Expression {
    // /// Get the type of this expression
    // pub fn r#type(&self) -> &crate::Type {
    //     match self {
    //         Self::Literal(literal) => literal.r#type(),
    //         Self::Function(function) => function.r#type(),
    //     }
    // }
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Literal(literal) => literal.fmt(f),
            Self::Variable(variable) => variable.read().unwrap().fmt(f),
            Self::Function(function) => function.fmt(f),
            Self::Call(call) => call.fmt(f),
            Self::Error => write!(f, "<ERROR>"),
        }
    }
}
