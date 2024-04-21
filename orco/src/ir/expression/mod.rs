use crate::{ir::Type, TypeInferenceInfo};

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
    /// Binary Operation
    BinaryOp(Box<Expression>, BinaryOp, Box<Expression>),
    /// Block expression, contains multiple expressions (something along { expr1; expr2; })
    Block(Block),
    /// Function call
    FunctionCall {
        /// Function name
        name: String,
        /// Arguments
        args: Vec<Expression>,
    },
    /// Return a value
    Return(Box<Expression>),
    /// Invalid expression
    Error,
}

impl Expression {
    /// Is this expression a block expression (f.e. a block, if statement, a for loop, etc.)
    pub fn is_block(&self) -> bool {
        match self {
            Expression::Block(_) => true,
            _ => false,
        }
    }

    /// Infer types
    pub fn infer_types(&mut self, target_type: &Type, type_inference: &TypeInferenceInfo) {
        match self {
            Expression::Constant(constant) => constant.infer_types(target_type),
            Expression::BinaryOp(lhs, _, rhs) => {
                lhs.infer_types(target_type, type_inference);
                rhs.infer_types(target_type, type_inference);
            }
            Expression::Block(block) => block.infer_types(target_type, type_inference),
            Expression::FunctionCall { name, args } => {
                if let Some(signature) = type_inference
                    .module
                    .items
                    .get(name)
                    .and_then(|item| item.function_signature())
                {
                    for (arg, signature_arg) in std::iter::zip(args, &signature.args) {
                        arg.infer_types(&signature_arg.1, type_inference);
                    }
                }
            }
            Expression::Return(expr) => {
                expr.infer_types(type_inference.return_type, type_inference)
            }
            Expression::Error => (),
        }
    }
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Constant(constant) => write!(f, "{}", constant),
            Expression::BinaryOp(lhs, op, rhs) => write!(f, "({} {} {})", lhs, op, rhs),
            Expression::Block(block) => write!(f, "{}", block),
            Expression::FunctionCall { name, args } => {
                write!(f, "{}(", name)?;
                for (index, arg) in args.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")?;
                Ok(())
            }
            Expression::Return(expr) => write!(f, "return {}", expr),
            Expression::Error => write!(f, "<ERROR>"),
        }
    }
}

/// Binary operators
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BinaryOp {
    /// Addition
    Add,
    /// Subtraction
    Sub,
    /// Multiplication
    Mul,
    /// Division
    Div,
    /// Modulo (Division Reminder)
    Mod,
}

impl std::fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Sub => write!(f, "-"),
            BinaryOp::Mul => write!(f, "*"),
            BinaryOp::Div => write!(f, "/"),
            BinaryOp::Mod => write!(f, "%"),
        }
    }
}
