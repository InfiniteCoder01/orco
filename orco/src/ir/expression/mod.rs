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
#[derive(Derivative)]
#[derivative(Debug)]
pub enum Expression {
    /// Function literal
    Function(Box<Function>),
    /// Extern function literal
    ExternFunction(ExternFunction),
    /// Module literal
    Module(Spanned<Module>),
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
        matches!(self, Self::Block(..) | Self::If(..))
    }

    /// Get the type this expression evaluates to
    pub fn get_type(&self) -> Type {
        match self {
            Self::Function(_) => Type::Function,
            Self::ExternFunction(_) => Type::ExternFunction,
            Self::Module(_) => Type::Module,
            Self::Constant(constant) => constant.get_type(),
            Self::Symbol(symbol, ..) => symbol.get_type(),
            Self::BinaryExpression(expr) => expr.get_type(),
            Self::UnaryExpression(expr) => expr.get_type(),
            Self::Block(block) => block.get_type(),
            Self::If(expr) => expr.get_type(),
            Self::Call(expr) => expr.get_type(),
            Self::Return(expr) => expr.get_type(),
            Self::VariableDeclaration(..) => Type::Unit,
            Self::Assignment(..) => Type::Unit,
            Self::Error(..) => Type::Error,
        }
    }

    /// Infer types
    /// Returns completed type of this expression (Completed means that type doesn't contain [`Type::Wildcard`], but rather [`Type::TypeVariable`])
    pub fn infer_types(&mut self, type_inference: &mut TypeInference) -> Type {
        let r#type = match self {
            Self::Function(_) => Type::Function,
            Self::ExternFunction(_) => Type::ExternFunction,
            Self::Module(_) => Type::Module,
            Self::Constant(constant) => constant.inner.infer_types(type_inference),
            Self::Symbol(symbol, meta) => symbol.infer_types(type_inference, meta.as_mut()),
            Self::BinaryExpression(expr) => expr.infer_types(type_inference),
            Self::UnaryExpression(expr) => expr.infer_types(type_inference),
            Self::Block(block) => block.infer_types(type_inference),
            Self::If(expr) => expr.infer_types(type_inference),
            Self::Call(expr) => expr.infer_types(type_inference),
            Self::Return(expr) => expr.infer_types(type_inference),
            Self::VariableDeclaration(declaration) => {
                declaration.as_ref().infer_types(type_inference)
            }
            Self::Assignment(expr) => expr.infer_types(type_inference),
            Self::Error(_) => Type::Error,
        };
        r#type
    }

    /// Finish types and check them
    pub fn finish_and_check_types(&mut self, type_inference: &mut TypeInference) -> Type {
        match self {
            Self::Function(_) => Type::Function,
            Self::ExternFunction(_) => Type::ExternFunction,
            Self::Module(_) => Type::Module,
            Self::Constant(constant) => constant
                .inner
                .finish_and_check_types(&constant.span, type_inference),
            Self::Symbol(symbol, metadata) => {
                symbol.finish_and_check_types(&symbol.span, type_inference, metadata.as_mut())
            }
            Self::BinaryExpression(expr) => expr.finish_and_check_types(type_inference),
            Self::UnaryExpression(expr) => expr.finish_and_check_types(type_inference),
            Self::Block(block) => block.finish_and_check_types(type_inference),
            Self::If(expr) => expr.finish_and_check_types(type_inference),
            Self::Call(expr) => expr.finish_and_check_types(type_inference),
            Self::Return(expr) => expr.finish_and_check_types(type_inference),
            Self::VariableDeclaration(declaration) => {
                declaration.finish_and_check_types(type_inference)
            }
            Self::Assignment(expr) => expr.finish_and_check_types(type_inference),
            Self::Error(_) => Type::Error,
        }
    }

    /// Get the span of this expression
    pub fn span(&self) -> Option<&Span> {
        match self {
            Self::Function(function) => function.span.as_ref(),
            Self::ExternFunction(function) => function.span.as_ref(),
            Self::Module(module) => module.span.as_ref(),
            Self::Constant(constant) => constant.span.as_ref(),
            Self::Symbol(symbol, ..) => symbol.span.as_ref(),
            Self::BinaryExpression(expr) => expr.span.as_ref(),
            Self::UnaryExpression(expr) => expr.span.as_ref(),
            Self::Block(block) => block.span.as_ref(),
            Self::If(expr) => expr.span.as_ref(),
            Self::Call(expr) => expr.span.as_ref(),
            Self::Return(expr) => expr.span.as_ref(),
            Self::VariableDeclaration(declaration) => declaration.span.as_ref(),
            Self::Assignment(expr) => expr.span.as_ref(),
            Self::Error(span) => span.as_ref(),
        }
    }
}

impl Clone for Expression {
    fn clone(&self) -> Self {
        match self {
            Self::Function(function) => Self::Function(function.clone()),
            Self::ExternFunction(function) => Self::ExternFunction(function.clone()),
            Self::Module(module) => Self::Module(module.clone()),
            Self::Constant(constant) => Self::Constant(constant.clone()),
            Self::Symbol(symbol, _) if !matches!(symbol.inner, SymbolReference::Unresolved(_)) => {
                unimplemented!(
                    "Cloning resolved symbols is not yet implemented, symbol: {}",
                    self
                )
            }
            Self::Symbol(symbol, meta) => Self::Symbol(symbol.clone(), meta.clone()),
            Self::BinaryExpression(expr) => Self::BinaryExpression(expr.clone()),
            Self::UnaryExpression(expr) => Self::UnaryExpression(expr.clone()),
            Self::Block(block) => Self::Block(block.clone()),
            Self::If(expr) => Self::If(expr.clone()),
            Self::Call(call) => Self::Call(call.clone()),
            Self::Return(expr) => Self::Return(expr.clone()),
            Self::VariableDeclaration(decl) => Self::VariableDeclaration(decl.clone()),
            Self::Assignment(expr) => Self::Assignment(expr.clone()),
            Self::Error(err) => Self::Error(err.clone()),
        }
    }
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Function(function) => write!(f, "{}", function),
            Self::ExternFunction(function) => write!(f, "{}", function),
            Self::Module(module) => write!(f, "{}", module),
            Self::Constant(constant) => write!(f, "{}", constant),
            Self::Symbol(symbol, ..) => write!(f, "{}", symbol),
            Self::BinaryExpression(expr) => write!(f, "{}", expr),
            Self::UnaryExpression(expr) => write!(f, "{}", expr),
            Self::Block(block) => write!(f, "{}", block),
            Self::If(expr) => write!(f, "{}", expr),
            Self::Call(expr) => write!(f, "{}", expr),
            Self::Return(expr) => write!(f, "{}", expr),
            Self::VariableDeclaration(declaration) => write!(f, "{}", declaration),
            Self::Assignment(expr) => write!(f, "{}", expr),
            Self::Error(_) => write!(f, "<ERROR>"),
        }
    }
}
