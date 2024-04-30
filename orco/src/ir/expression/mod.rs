use crate::diagnostics::*;
use crate::{ir::Type, type_inference::TypeInference};
use std::sync::{Arc, Mutex};

/// Constant value
pub mod constant;
pub use constant::Constant;

/// Code block
pub mod block;
pub use block::Block;

/// Variable declaration
pub mod variable_declaration;
pub use variable_declaration::VariableDeclaration;
pub use variable_declaration::VariableReference;

/// An expression
#[derive(Clone, Debug)]
pub enum Expression {
    /// A constant value
    Constant(Spanned<Constant>),
    /// Variable
    Variable(Spanned<VariableReference>),
    /// Binary Operation
    BinaryOp(Box<Expression>, BinaryOp, Box<Expression>),
    /// Block expression, contains multiple expressions (something along { expr1; expr2; })
    Block(Spanned<Block>),
    /// Function call
    FunctionCall {
        /// Function name
        name: Spanned<String>,
        /// Arguments
        args: Spanned<Vec<Expression>>,
    },
    /// Return a value
    Return(Spanned<Box<Expression>>),
    /// Declare a variable
    VariableDeclaration(VariableReference),
    /// Invalid expression
    Error(Span),
}

impl Expression {
    /// Is this expression a block expression (f.e. a block, if statement, a for loop, etc.)
    pub fn is_block(&self) -> bool {
        matches!(self, Expression::Block(_))
    }

    /// Get the type this expression evaluates to
    pub fn get_type(&self, root: &crate::ir::Module) -> Type {
        match self {
            Expression::Constant(constant) => constant.get_type(),
            Expression::Variable(variable) => variable.lock().unwrap().r#type.inner.clone(),
            Expression::BinaryOp(lhs, _, rhs) => lhs.get_type(root) | rhs.get_type(root),
            Expression::Block(block) => block.get_type(root),
            Expression::FunctionCall { name, .. } => {
                if let Some(signature) = root
                    .items
                    .get(&name.inner)
                    .and_then(|item| item.function_signature())
                {
                    (*signature.return_type).clone()
                } else {
                    Type::Error
                }
            }
            Expression::Return(_) => Type::Never,
            Expression::VariableDeclaration { .. } => Type::Unit,
            Expression::Error(_) => Type::Error,
        }
    }

    /// Infer types
    /// target_type should be [`Type::Wildcard`] when unknown
    /// Never returns a [`Type::Wildcard`]. Only [`Type::TypeVariable`]
    pub fn infer_types(&mut self, target_type: &Type, type_inference: &mut TypeInference) -> Type {
        let r#type = match self {
            Expression::Constant(constant) => {
                constant.inner.infer_types(target_type, type_inference)
            }
            Expression::Variable(variable) => {
                let reference = variable.lock().unwrap();
                type_inference.equate(target_type, &reference.r#type.inner)
            }
            Expression::BinaryOp(lhs, _, rhs) => {
                let lhs_type = lhs.infer_types(target_type, type_inference);
                let rhs_type = rhs.infer_types(target_type, type_inference);
                type_inference.equate(&lhs_type, &rhs_type)
            }
            Expression::Block(block) => block.infer_types(target_type, type_inference),
            Expression::FunctionCall { name, args } => {
                if let Some(signature) = type_inference.signature(name) {
                    for (arg, signature_arg) in std::iter::zip(&mut args.inner, &signature.args) {
                        arg.infer_types(&signature_arg.1, type_inference);
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
            Expression::BinaryOp(lhs, op, rhs) => {
                let lhs_type = lhs.finish_and_check_types(type_inference);
                let rhs_type = rhs.finish_and_check_types(type_inference);
                if !rhs_type.morphs(&lhs_type) {
                    let mut colors = ColorGenerator::new();
                    let report =
                        Report::build(ReportKind::Error, lhs.span().0.clone(), lhs.span().1.start)
                            .with_message(format!(
                                "Incompatible types for binary operation '{}': {} and {}",
                                op, lhs_type, rhs_type
                            ))
                            .with_label(
                                Label::new(lhs.span().clone())
                                    .with_message("Left hand side")
                                    .with_color(colors.next()),
                            )
                            .with_label(
                                Label::new(rhs.span().clone())
                                    .with_message("Right hand side")
                                    .with_color(colors.next()),
                            );
                    type_inference.reporter.report(report.finish());
                    todo!(
                        "Type mismatch for binary operator error: {:?} and {:?}",
                        lhs_type,
                        rhs_type
                    );
                }
                lhs_type | rhs_type
            }
            Expression::Block(block) => block.finish_and_check_types(type_inference),
            Expression::FunctionCall { name, args } => {
                if let Some(signature) = type_inference.signature(name) {
                    if args.inner.len() != signature.args.len() {
                        type_inference.reporter.report_type_error(
                            format!(
                                "Argument count mismatch: Function '{}' expects {} arguments, but {} were given",
                                name.inner,
                                signature.args.len(),
                                args.inner.len()
                            ),
                            args.span.clone(),
                            None
                        );
                    }
                    for (arg, signature_arg) in std::iter::zip(&mut args.inner, &signature.args) {
                        let arg_type = arg.finish_and_check_types(type_inference);
                        if !arg_type.morphs(&signature_arg.1) {
                            type_inference.reporter.report_type_error(
                                format!(
                                    "Incompatible argument types for function '{}': expected '{}', got '{}'",
                                    name.inner,
                                    arg_type, signature_arg.1.inner
                                ),
                                arg.span(),
                                Some(signature_arg.1.span.clone()),
                            );
                        }
                    }
                    (*signature.return_type).clone()
                } else {
                    type_inference.reporter.report_type_error(
                        format!("Function '{}' was not found in this scope", name.inner),
                        name.span.extend(&args.span),
                        None,
                    );
                    Type::Error
                }
            }
            Expression::Return(expr) => {
                let r#type = expr.finish_and_check_types(type_inference);
                if !r#type.morphs(type_inference.return_type) {
                    type_inference.reporter.report_type_error(
                        format!(
                            "Return type mismatch: expected {}, got {}",
                            type_inference.return_type.inner, r#type
                        ),
                        expr.span(),
                        Some(type_inference.return_type.span.clone()),
                    );
                }
                Type::Never
            }
            Expression::VariableDeclaration(variable_declaration) => variable_declaration
                .lock()
                .unwrap()
                .finish_and_check_types(type_inference),
            Expression::Error(_) => Type::Error,
        };
        r#type
    }

    /// Get the span of this expression
    pub fn span(&self) -> Span {
        match self {
            Expression::Constant(constant) => constant.span.clone(),
            Expression::Variable(variable) => variable.span.clone(),
            Expression::BinaryOp(lhs, _, rhs) => lhs.span().extend(&rhs.span()),
            Expression::Block(block) => block.span.clone(),
            Expression::FunctionCall { name, args } => name.span.extend(&args.span),
            Expression::Return(expr) => expr.span.clone(),
            Expression::VariableDeclaration(variable_declaration) => {
                variable_declaration.span.clone()
            }
            Expression::Error(span) => span.clone(),
        }
    }
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Constant(constant) => write!(f, "{}", constant.inner),
            Expression::Variable(variable) => write!(f, "{}", variable.lock().unwrap().name.inner),
            Expression::BinaryOp(lhs, op, rhs) => write!(f, "({} {} {})", lhs, op, rhs),
            Expression::Block(block) => write!(f, "{}", block.inner),
            Expression::FunctionCall { name, args } => {
                write!(f, "{}(", name.inner)?;
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
                write!(f, "{}", variable_declaration.lock().unwrap())
            }
            Expression::Error(_) => write!(f, "<ERROR>"),
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
