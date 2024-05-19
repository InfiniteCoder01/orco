use super::*;
use diagnostics::*;
use ir::Type;
use symbol_reference::SymbolReference;
use type_inference::TypeInference;

/// Constant value
pub mod constant;
pub use constant::Constant;

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

/// Call expression (function call)
pub mod call;
pub use call::CallExpression;

/// Variable declaration
pub mod variable_declaration;
pub use variable_declaration::Variable;
pub use variable_declaration::VariableDeclaration;

/// An expression
#[derive(Clone, Debug)]
pub enum Expression {
    /// A constant value
    Constant(Spanned<Constant>),
    /// Variable
    Symbol(Spanned<SymbolReference>),
    /// Binary expression
    BinaryExpression(Spanned<BinaryExpression>),
    /// Unary expression
    UnaryExpression(Spanned<UnaryExpression>),
    /// Block expression, contains multiple expressions (something along { expr1; expr2; })
    Block(Spanned<Block>),
    /// If expression (and ternary operator)
    If(Spanned<IfExpression>),
    // /// While loop
    // While {
    //     /// Condition
    //     condition: Box<Expression>,
    //     /// Body
    //     body: Spanned<Block>,
    //     /// Span
    //     span: Span,
    // },
    /// Function call
    Call(Spanned<CallExpression>),
    /// Return a value
    Return(Spanned<Box<Expression>>),
    /// Declare a variable
    VariableDeclaration(Variable),
    /// Assignment
    Assignment(Spanned<AssignmentExpression>),
    /// Invalid expression
    Error(Span),
}

impl Expression {
    /// Is this expression a block expression (f.e. a block, if statement, a for loop, etc.)
    pub fn is_block(&self) -> bool {
        matches!(
            self,
            Expression::Block(..) | Expression::If(..) // | Expression::While { .. }
        )
    }

    /// Get the type this expression evaluates to
    pub fn get_type(&self) -> Type {
        match self {
            Expression::Constant(constant) => constant.get_type(),
            Expression::Symbol(symbol) => symbol.get_type(),
            Expression::BinaryExpression(expr) => expr.get_type(),
            Expression::UnaryExpression(expr) => expr.get_type(),
            Expression::Block(block) => block.get_type(),
            Expression::If(expr) => expr.get_type(),
            // Expression::While { .. } => Type::unit(),
            Expression::Call(expr) => expr.get_type(),
            Expression::Return(..) => Type::Never,
            Expression::VariableDeclaration(..) => Type::Unit,
            Expression::Assignment(..) => Type::Unit,
            Expression::Error(..) => Type::Error,
        }
    }

    /// Infer types
    /// Returns completed type of this expression (Completed means that type doesn't contain [`Type::Wildcard`], but rather [`Type::TypeVariable`])
    pub fn infer_types(&mut self, type_inference: &mut TypeInference) -> Type {
        let r#type = match self {
            Expression::Constant(constant) => constant.inner.infer_types(type_inference),
            Expression::Symbol(symbol) => symbol.infer_types(type_inference),
            Expression::BinaryExpression(expr) => expr.infer_types(type_inference),
            Expression::UnaryExpression(expr) => expr.infer_types(type_inference),
            Expression::Block(block) => block.infer_types(type_inference),
            Expression::If(expr) => expr.infer_types(type_inference),
            // Expression::While {
            //     condition, body, ..
            // } => {
            //     condition.infer_types(&Type::Bool, type_inference);
            //     body.infer_types(type_inference);
            //     Type::unit()
            // }
            Expression::Call(expr) => expr.infer_types(type_inference),
            Expression::Return(expr) => {
                let r#type = expr.infer_types(type_inference);
                type_inference.equate(&r#type, &type_inference.return_type);
                Type::Never
            }
            Expression::VariableDeclaration(declaration) => {
                let r#type = declaration.infer_types(type_inference);
                type_inference.current_scope_mut().insert(
                    declaration.name.clone(),
                    SymbolReference::Variable(declaration.clone()),
                );
                r#type
            }
            Expression::Assignment(expr) => expr.infer_types(type_inference),
            Expression::Error(_) => Type::Error,
        };
        r#type
    }

    /// Finish types and check them
    pub fn finish_and_check_types(&mut self, type_inference: &mut TypeInference) -> Type {
        let r#type = match self {
            Expression::Constant(constant) => constant
                .inner
                .finish_and_check_types(constant.span.clone(), type_inference),
            Expression::Symbol(symbol) => {
                symbol.finish_and_check_types(symbol.span.clone(), type_inference)
            }
            Expression::BinaryExpression(expr) => expr.finish_and_check_types(type_inference),
            Expression::UnaryExpression(expr) => expr.finish_and_check_types(type_inference),
            Expression::Block(block) => block.finish_and_check_types(type_inference),
            Expression::If(expr) => expr.finish_and_check_types(type_inference),
            // Expression::While {
            //     condition, body, ..
            // } => {
            //     let condition_type = condition.finish_and_check_types(type_inference);
            //     if !condition_type.morphs(&Type::Bool) {
            //         type_inference.reporter.report_type_error(
            //             format!(
            //                 "If condition should be of type 'bool', but it is of type '{}'",
            //                 condition_type
            //             ),
            //             condition.span(),
            //             vec![],
            //         );
            //     }
            //     let body_type = body.finish_and_check_types(type_inference);
            //     if !body_type.morphs(&Type::Unit) {
            //         type_inference.reporter.report_type_error(
            //             format!(
            //                 "While body should be of type 'bool', but it is of type '{}'",
            //                 body_type
            //             ),
            //             body.span.clone(),
            //             vec![],
            //         );
            //     }
            //     Type::unit()
            // }
            Expression::Call(expr) => expr.finish_and_check_types(type_inference),
            Expression::Return(expr) => {
                let r#type = expr.finish_and_check_types(type_inference);
                if !r#type.morphs(type_inference.return_type) {
                    type_inference.reporter.report_type_error(
                        format!(
                            "Return type mismatch: expected '{}', got '{}'",
                            type_inference.return_type.inner, r#type
                        ),
                        expr.span(),
                        vec![(
                            "Expected because of this",
                            type_inference.return_type.span.clone(),
                        )],
                    );
                }
                Type::Never
            }
            Expression::VariableDeclaration(declaration) => {
                declaration.finish_and_check_types(type_inference)
            }
            Expression::Assignment(expr) => expr.finish_and_check_types(type_inference),
            Expression::Error(_) => Type::Error,
        };
        r#type
    }

    /// Get the span of this expression
    pub fn span(&self) -> Span {
        match self {
            Expression::Constant(constant) => constant.span.clone(),
            Expression::Symbol(symbol) => symbol.span.clone(),
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
            Expression::Constant(constant) => write!(f, "{}", constant.inner),
            Expression::Symbol(symbol) => write!(f, "{}", symbol.inner),
            Expression::BinaryExpression(expr) => write!(f, "{}", expr.inner),
            Expression::UnaryExpression(expr) => write!(f, "{}", expr.inner),
            Expression::Block(block) => write!(f, "{}", block.inner),
            Expression::If(expr) => write!(f, "{}", expr.inner),
            Expression::Call(expr) => write!(f, "{}", expr.inner),
            Expression::Return(expr) => write!(f, "return {}", expr.inner),
            Expression::VariableDeclaration(declaration) => {
                write!(f, "{}", declaration.inner)
            }
            Expression::Assignment(expr) => write!(f, "{}", expr.inner),
            Expression::Error(_) => write!(f, "<ERROR>"),
        }
    }
}
