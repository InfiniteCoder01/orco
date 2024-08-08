use super::*;
use ir::Type;
use type_inference::TypeInference;

/// Function
pub mod function;
pub use function::ExternFunction;
pub use function::Function;

/// Constant value
pub mod constant;
pub use constant::Constant;

/// Symbol reference
pub mod symbol_reference;
pub use symbol_reference::SymbolReference;

/// Operator-oriented expressions (binary, unary, assignment, etc.)
pub mod operator;
pub use operator::AssignmentExpression;
pub use operator::BinaryExpression;
pub use operator::BinaryOp;
pub use operator::UnaryExpression;
pub use operator::UnaryOp;

/// Code block
pub mod block;
pub use block::Block;

/// Branching constructs
pub mod branching;
pub use branching::IfExpression;

/// Control flow
pub mod control_flow;
pub use control_flow::ReturnExpression;

/// Call expression (function call)
pub mod call;
pub use call::CallExpression;

/// Variable declaration
pub mod variable_declaration;
pub use variable_declaration::VariableDeclaration;

/// An expression
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub enum Expression {
    /// Function literal
    Function(Box<Function>),
    /// Extern function literal
    ExternFunction(ExternFunction),
    /// A constant value
    Constant(Spanned<Constant>),
    /// Symbol
    Symbol(
        Spanned<SymbolReference>,
        #[derivative(Debug = "ignore")] Box<dyn symbol_reference::SymbolMetadata>,
    ),
    /// Binary expression
    BinaryExpression(BinaryExpression),
    /// Unary expression
    UnaryExpression(UnaryExpression),
    /// Block expression, contains multiple expressions (something along { expr1; expr2; })
    Block(Block),
    /// If expression (and ternary operator)
    If(IfExpression),
    /// Function call
    Call(CallExpression),
    /// Return a value
    Return(ReturnExpression),
    /// Declare a variable
    VariableDeclaration(std::pin::Pin<Box<VariableDeclaration>>),
    /// Assignment
    Assignment(AssignmentExpression),
    /// Invalid expression
    Error(Option<Span>),
}

impl Expression {
    /// Is this expression a block expression (f.e. a block, if statement, a for loop, etc.)
    pub fn is_block(&self) -> bool {
        matches!(self, Expression::Block(..) | Expression::If(..))
    }

    /// Get the type this expression evaluates to
    pub fn get_type(&self) -> Type {
        match self {
            Expression::Function(_) => Type::Function,
            Expression::ExternFunction(_) => Type::ExternFunction,
            Expression::Constant(constant) => constant.get_type(),
            Expression::Symbol(symbol, ..) => symbol.get_type(),
            Expression::BinaryExpression(expr) => expr.get_type(),
            Expression::UnaryExpression(expr) => expr.get_type(),
            Expression::Block(block) => block.get_type(),
            Expression::If(expr) => expr.get_type(),
            Expression::Call(expr) => expr.get_type(),
            Expression::Return(expr) => expr.get_type(),
            Expression::VariableDeclaration(..) => Type::Unit,
            Expression::Assignment(..) => Type::Unit,
            Expression::Error(..) => Type::Error,
        }
    }

    /// Infer types
    /// Returns completed type of this expression (Completed means that type doesn't contain [`Type::Wildcard`], but rather [`Type::TypeVariable`])
    pub fn infer_types(&mut self, type_inference: &mut TypeInference) -> Type {
        let r#type = match self {
            Expression::Function(_) => Type::Function,
            Expression::ExternFunction(_) => Type::ExternFunction,
            Expression::Constant(constant) => constant.inner.infer_types(type_inference),
            Expression::Symbol(symbol, metadata) => {
                symbol.infer_types(type_inference, metadata.as_mut())
            }
            Expression::BinaryExpression(expr) => expr.infer_types(type_inference),
            Expression::UnaryExpression(expr) => expr.infer_types(type_inference),
            Expression::Block(block) => block.infer_types(type_inference),
            Expression::If(expr) => expr.infer_types(type_inference),
            Expression::Call(expr) => expr.infer_types(type_inference),
            Expression::Return(expr) => expr.infer_types(type_inference),
            Expression::VariableDeclaration(declaration) => {
                declaration.as_ref().infer_types(type_inference)
            }
            Expression::Assignment(expr) => expr.infer_types(type_inference),
            Expression::Error(_) => Type::Error,
        };
        r#type
    }

    /// Finish types and check them
    pub fn finish_and_check_types(&mut self, type_inference: &mut TypeInference) -> Type {
        match self {
            Expression::Function(_) => Type::Function,
            Expression::ExternFunction(_) => Type::ExternFunction,
            Expression::Constant(constant) => constant
                .inner
                .finish_and_check_types(constant.span.clone(), type_inference),
            Expression::Symbol(symbol, metadata) => symbol.finish_and_check_types(
                symbol.span.clone(),
                type_inference,
                metadata.as_mut(),
            ),
            Expression::BinaryExpression(expr) => expr.finish_and_check_types(type_inference),
            Expression::UnaryExpression(expr) => expr.finish_and_check_types(type_inference),
            Expression::Block(block) => block.finish_and_check_types(type_inference),
            Expression::If(expr) => expr.finish_and_check_types(type_inference),
            Expression::Call(expr) => expr.finish_and_check_types(type_inference),
            Expression::Return(expr) => expr.finish_and_check_types(type_inference),
            Expression::VariableDeclaration(declaration) => {
                declaration.finish_and_check_types(type_inference)
            }
            Expression::Assignment(expr) => expr.finish_and_check_types(type_inference),
            Expression::Error(_) => Type::Error,
        }
    }

    /// Get the span of this expression
    pub fn span(&self) -> Option<Span> {
        match self {
            Expression::Function(function) => function.span.clone(),
            Expression::ExternFunction(function) => function.span.clone(),
            Expression::Constant(constant) => constant.span.clone(),
            Expression::Symbol(symbol, ..) => symbol.span.clone(),
            Expression::BinaryExpression(expr) => expr.span.clone(),
            Expression::UnaryExpression(expr) => expr.span.clone(),
            Expression::Block(block) => block.span.clone(),
            Expression::If(expr) => expr.span.clone(),
            Expression::Call(expr) => expr.span.clone(),
            Expression::Return(expr) => expr.span.clone(),
            Expression::VariableDeclaration(declaration) => declaration.span.clone(),
            Expression::Assignment(expr) => expr.span.clone(),
            Expression::Error(span) => span.clone(),
        }
    }
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Function(function) => write!(f, "{}", function),
            Expression::ExternFunction(function) => write!(f, "{}", function),
            Expression::Constant(constant) => write!(f, "{}", constant.inner),
            Expression::Symbol(symbol, ..) => write!(f, "{}", symbol.inner),
            Expression::BinaryExpression(expr) => write!(f, "{}", expr),
            Expression::UnaryExpression(expr) => write!(f, "{}", expr),
            Expression::Block(block) => write!(f, "{}", block),
            Expression::If(expr) => write!(f, "{}", expr),
            Expression::Call(expr) => write!(f, "{}", expr),
            Expression::Return(expr) => write!(f, "{}", expr),
            Expression::VariableDeclaration(declaration) => write!(f, "{}", declaration),
            Expression::Assignment(expr) => write!(f, "{}", expr),
            Expression::Error(_) => write!(f, "<ERROR>"),
        }
    }
}
