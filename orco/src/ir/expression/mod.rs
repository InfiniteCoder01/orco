use crate::diagnostics::*;
use crate::{ir::Type, type_inference::TypeInference};
use std::sync::{Arc, Mutex};

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

/// Variable declaration
pub mod variable_declaration;
pub use variable_declaration::VariableDeclaration;
pub use variable_declaration::VariableReference;
pub use variable_declaration::VariableReferenceExt;

/// An expression
#[derive(Clone, Debug)]
pub enum Expression {
    /// A constant value
    Constant(Spanned<Constant>),
    /// Variable
    Variable(Spanned<VariableReference>),
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
    FunctionCall {
        /// Function name
        name: Span,
        /// Arguments
        args: Spanned<Vec<Expression>>,
    },
    /// Return a value
    Return(Spanned<Box<Expression>>),
    /// Declare a variable
    VariableDeclaration(VariableReference),
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
            Expression::Block(_) | Expression::If { .. } // | Expression::While { .. }
        )
    }

    /// Get the type this expression evaluates to
    pub fn get_type(&self, root: &crate::ir::Module) -> Type {
        match self {
            Expression::Constant(constant) => constant.get_type(),
            Expression::Variable(variable) => variable.lock().unwrap().r#type.inner.clone(),
            Expression::BinaryExpression(expr) => expr.get_type(root),
            Expression::UnaryExpression(expr) => expr.get_type(root),
            Expression::Block(block) => block.get_type(root),
            Expression::If(expr) => expr.get_type(root),
            // Expression::While { .. } => Type::unit(),
            Expression::FunctionCall { name, .. } => {
                if let Some(signature) = root
                    .symbols
                    .get(name)
                    .and_then(|symbol| symbol.function_signature())
                {
                    (*signature.return_type).clone()
                } else {
                    Type::Error
                }
            }
            Expression::Return(..) => Type::Never,
            Expression::VariableDeclaration(..) => Type::Unit,
            Expression::Assignment(..) => Type::Unit,
            Expression::Error(..) => Type::Error,
        }
    }

    /// Infer types
    /// target_type should be [`Type::Wildcard`] when unknown. infer_types doesn't strictly enforce,
    /// that value will be of target_type, so you should do it manually
    /// Never returns a [`Type::Wildcard`]. Only [`Type::TypeVariable`]
    pub fn infer_types(&mut self, target_type: &Type, type_inference: &mut TypeInference) -> Type {
        let r#type = match self {
            Expression::Constant(constant) => {
                constant.inner.infer_types(target_type, type_inference)
            }
            Expression::Variable(variable) => {
                type_inference.equate(target_type, &variable.r#type())
            }
            Expression::BinaryExpression(expr) => expr.infer_types(target_type, type_inference),
            Expression::UnaryExpression(expr) => expr.infer_types(target_type, type_inference),
            Expression::Block(block) => block.infer_types(target_type, type_inference),
            Expression::If(expr) => expr.infer_types(target_type, type_inference),
            // Expression::While {
            //     condition, body, ..
            // } => {
            //     condition.infer_types(&Type::Bool, type_inference);
            //     body.infer_types(target_type, type_inference);
            //     Type::unit()
            // }
            Expression::FunctionCall { name, args } => {
                if let Some(signature) = type_inference.signature(name) {
                    for (arg, signature_arg) in
                        std::iter::zip(&mut args.inner, &signature.args.inner)
                    {
                        arg.infer_types(&signature_arg.r#type(), type_inference);
                    }
                    (*signature.return_type).clone()
                } else {
                    Type::Error
                }
            }
            Expression::Return(expr) => {
                expr.infer_types(type_inference.return_type, type_inference);
                Type::Never
            }
            Expression::VariableDeclaration(variable_declaration) => variable_declaration
                .lock()
                .unwrap()
                .infer_types(type_inference),
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
            Expression::Variable(variable) => variable.lock().unwrap().r#type.inner.clone(),
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
            Expression::FunctionCall { name, args } => {
                if let Some(signature) = type_inference.signature(name) {
                    if args.inner.len() != signature.args.len() {
                        type_inference.reporter.report_type_error(
                            format!(
                                "Argument count mismatch: Function '{}' expects {} arguments, but {} were given",
                                name,
                                signature.args.len(),
                                args.inner.len()
                            ),
                            args.span.clone(),
                            vec![("Expected because of this", signature.args.span.clone())], 
                        );
                    }
                    for (arg, signature_arg) in
                        std::iter::zip(&mut args.inner, &signature.args.inner)
                    {
                        let arg_type = arg.finish_and_check_types(type_inference);
                        let signature_arg = signature_arg.lock().unwrap();
                        if !arg_type.morphs(&signature_arg.r#type) {
                            type_inference.reporter.report_type_error(
                                format!(
                                    "Incompatible argument types for function '{}': expected '{}', got '{}'",
                                    name,
                                    arg_type, signature_arg.r#type.inner
                                ),
                                arg.span(),
                                vec![("Expected because of this", signature_arg.r#type.span.clone())],
                            );
                        }
                    }
                    (*signature.return_type).clone()
                } else {
                    type_inference.reporter.report_type_error(
                        format!("Function '{}' was not found in this scope", name),
                        name.extend(&args.span),
                        vec![],
                    );
                    Type::Error
                }
            }
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
            Expression::VariableDeclaration(variable_declaration) => variable_declaration
                .lock()
                .unwrap()
                .finish_and_check_types(type_inference),
            Expression::Assignment(expr) => expr.finish_and_check_types(type_inference),
            Expression::Error(_) => Type::Error,
        };
        r#type
    }

    /// Get the span of this expression
    pub fn span(&self) -> Span {
        match self {
            Expression::Constant(constant) => constant.span.clone(),
            Expression::Variable(variable) => variable.span.clone(),
            Expression::BinaryExpression(expr) => expr.span.clone(),
            Expression::UnaryExpression(expr) => expr.span.clone(),
            Expression::Block(block) => block.span.clone(),
            Expression::If(expr) => expr.span.clone(),
            Expression::FunctionCall { name, args } => name.extend(&args.span),
            Expression::Return(expr) => expr.span.clone(),
            Expression::VariableDeclaration(variable_declaration) => {
                variable_declaration.span.clone()
            }
            Expression::Assignment(expr) => expr.span.clone(),
            Expression::Error(span) => span.clone(),
        }
    }
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Constant(constant) => constant.fmt(f),
            Expression::Variable(variable) => {
                let variable = variable.lock().unwrap();
                let show_id =
                    std::env::var("ORCO_SHOW_VAR_ID").map_or(false, |show_id| show_id == "1");
                if show_id {
                    write!(f, "{} (#{})", variable.name, variable.id)
                } else {
                    write!(f, "{}", variable.name)
                }
            }
            Expression::BinaryExpression(expr) => expr.fmt(f),
            Expression::UnaryExpression(expr) => expr.fmt(f),
            Expression::Block(block) => block.fmt(f),
            Expression::If(expr) => expr.fmt(f),
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
            Expression::Return(expr) => write!(f, "return {}", expr.inner),
            Expression::VariableDeclaration(variable_declaration) => {
                variable_declaration.lock().unwrap().fmt(f)
            }
            Expression::Assignment(expr) => expr.fmt(f),
            Expression::Error(_) => write!(f, "<ERROR>"),
        }
    }
}
