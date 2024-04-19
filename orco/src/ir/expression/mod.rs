use crate::ir::Type;

/// Constant value
pub mod constant;
pub use constant::Constant;

/// Code block
pub mod block;
pub use block::Block;

/// An expression
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Expression {
    /// A constant value
    Constant(Constant),
    /// Block expression, contains multiple expressions (something along { expr1; expr2; })
    Block(Block),
    /// Return a value
    Return(Box<Expression>),
    /// Function call
    FunctionCall {
        /// Function name
        name: String,
    },
}

impl Expression {
    /// Infer types
    pub fn infer_types(&mut self, target_type: &Type, return_type: &Type) {
        match self {
            Expression::Constant(constant) => constant.infer_types(target_type),
            Expression::Block(block) => block.infer_types(target_type, return_type),
            Expression::Return(expr) => expr.infer_types(return_type, return_type),
            Expression::FunctionCall { .. } => (),
        }
    }
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Constant(constant) => write!(f, "{}", constant),
            Expression::Block(block) => write!(f, "{}", block),
            Expression::Return(expr) => write!(f, "return {}", expr),
            Expression::FunctionCall { name } => write!(f, "{}()", name),
        }
    }
}
