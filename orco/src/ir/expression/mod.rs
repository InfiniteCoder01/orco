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
pub use variable_declaration::VariableReferenceExt;

/// An expression
#[derive(Clone, Debug)]
pub enum Expression {
    /// A constant value
    Constant(Spanned<Constant>),
    /// Variable
    Variable(Spanned<VariableReference>),
    /// Binary Operation
    BinaryOp(Box<Expression>, BinaryOp, Box<Expression>),
    /// Unary Operation
    UnaryOp(Spanned<UnaryOp>, Box<Expression>),
    /// Block expression, contains multiple expressions (something along { expr1; expr2; })
    Block(Spanned<Block>),
    /// If expression (and ternary operator)
    If {
        /// Condition
        condition: Box<Expression>,
        /// Then branch
        then_branch: Box<Expression>,
        /// Else branch
        else_branch: Option<Box<Expression>>,
        /// Span
        span: Span,
    },
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
        name: Spanned<String>,
        /// Arguments
        args: Spanned<Vec<Expression>>,
    },
    /// Return a value
    Return(Spanned<Box<Expression>>),
    /// Declare a variable
    VariableDeclaration(VariableReference),
    /// Assignment
    Assignment(Box<Expression>, Box<Expression>),
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
            Expression::BinaryOp(lhs, op, rhs) => match op {
                BinaryOp::Eq
                | BinaryOp::Ne
                | BinaryOp::Lt
                | BinaryOp::Le
                | BinaryOp::Gt
                | BinaryOp::Ge => Type::Bool,
                _ => lhs.get_type(root) | rhs.get_type(root),
            },
            Expression::UnaryOp(_, expr) => expr.get_type(root),
            Expression::Block(block) => block.get_type(root),
            Expression::If {
                then_branch,
                else_branch,
                ..
            } => else_branch.as_ref().map_or_else(
                || Type::Unit,
                |else_branch| then_branch.get_type(root) | else_branch.get_type(root),
            ),
            // Expression::While { .. } => Type::unit(),
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
            Expression::VariableDeclaration(_) => Type::Unit,
            Expression::Assignment(..) => Type::Unit,
            Expression::Error(_) => Type::Error,
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
            Expression::BinaryOp(lhs, op, rhs) => match op {
                BinaryOp::Eq
                | BinaryOp::Ne
                | BinaryOp::Lt
                | BinaryOp::Le
                | BinaryOp::Gt
                | BinaryOp::Ge => {
                    let lhs_type = lhs.infer_types(&Type::Wildcard, type_inference);
                    let rhs_type = rhs.infer_types(&lhs_type, type_inference);
                    type_inference.equate(&lhs_type, &rhs_type);
                    Type::Bool
                }
                _ => {
                    let lhs_type = lhs.infer_types(target_type, type_inference);
                    let rhs_type = rhs.infer_types(target_type, type_inference);
                    type_inference.equate(&lhs_type, &rhs_type)
                }
            },
            Expression::UnaryOp(_, expr) => expr.infer_types(target_type, type_inference),
            Expression::Block(block) => block.infer_types(target_type, type_inference),
            Expression::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                condition.infer_types(&Type::Bool, type_inference);
                let then_type = then_branch.infer_types(target_type, type_inference);
                if let Some(else_branch) = else_branch {
                    let else_type = else_branch.infer_types(target_type, type_inference);
                    type_inference.equate(&then_type, &else_type)
                } else {
                    Type::unit()
                }
            }
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
            Expression::Assignment(target, value) => {
                let value = value.infer_types(&Type::Wildcard, type_inference);
                let target = target.infer_types(&value, type_inference);
                type_inference.equate(&target, &value);
                Type::Unit
            }
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
                                "Incompatible types for binary operation '{}': '{}' and '{}'",
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
                match op {
                    BinaryOp::Eq
                    | BinaryOp::Ne
                    | BinaryOp::Lt
                    | BinaryOp::Le
                    | BinaryOp::Gt
                    | BinaryOp::Ge => Type::Bool,
                    _ => lhs_type | rhs_type,
                }
            }
            Expression::UnaryOp(op, expr) => {
                let r#type = expr.finish_and_check_types(type_inference);
                match op.inner {
                    UnaryOp::Neg => match r#type {
                        Type::Int(_) => (),
                        _ => {
                            type_inference.reporter.report_type_error(
                                format!("Cannot apply unary negation to type '{}'", r#type),
                                expr.span(),
                                vec![],
                            );
                        }
                    },
                }
                r#type
            }
            Expression::Block(block) => block.finish_and_check_types(type_inference),
            Expression::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                let condition_type = condition.finish_and_check_types(type_inference);
                if !condition_type.morphs(&Type::Bool) {
                    type_inference.reporter.report_type_error(
                        format!(
                            "If condition should be of type 'bool', but it is of type '{}'",
                            condition_type
                        ),
                        condition.span(),
                        vec![],
                    );
                }
                let then_type = then_branch.finish_and_check_types(type_inference);
                if let Some(else_branch) = else_branch {
                    let else_type = else_branch.finish_and_check_types(type_inference);
                    if !else_type.morphs(&then_type) {
                        type_inference.reporter.report_type_error(
                            format!(
                                "Else branch type mismatch: Expected '{}', got '{}'",
                                then_type, else_type
                            ),
                            else_branch.span().clone(),
                            vec![("Expected because of this", then_branch.span().clone())],
                        );
                    }
                    then_type
                } else {
                    Type::unit()
                }
            }
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
                                name.inner,
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
                                    name.inner,
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
                        format!("Function '{}' was not found in this scope", name.inner),
                        name.span.extend(&args.span),
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
            Expression::Assignment(target, value) => {
                let value_type = value.finish_and_check_types(type_inference);
                let target_type = target.finish_and_check_types(type_inference);
                match target.as_ref() {
                    Expression::Variable(variable) => {
                        let variable = variable.lock().unwrap();
                        if !variable.mutable.inner {
                            type_inference.reporter.report_type_error(
                                format!(
                                    "Cannot assign to an immutable variable '{}'",
                                    variable.name.inner
                                ),
                                target.span(),
                                vec![(
                                    "Help: Make this variable mutable",
                                    variable.mutable.span.clone(),
                                )],
                            )
                        }
                    }
                    _ => type_inference.reporter.report_type_error(
                        format!("Cannot assign to '{}'", target),
                        target.span(),
                        vec![],
                    ),
                };
                if !value_type.morphs(&target_type) {
                    type_inference.reporter.report_type_error(
                        format!("Cannot assign '{}' to '{}'", value_type, target_type),
                        value.span(),
                        vec![("Expected because of this", target.span())],
                    );
                }
                Type::Unit
            }
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
            Expression::UnaryOp(op, expr) => op.span.extend(&expr.span()),
            Expression::Block(block) => block.span.clone(),
            Expression::If { span, .. } => span.clone(),
            Expression::FunctionCall { name, args } => name.span.extend(&args.span),
            Expression::Return(expr) => expr.span.clone(),
            Expression::VariableDeclaration(variable_declaration) => {
                variable_declaration.span.clone()
            }
            Expression::Assignment(target, value) => target.span().extend(&value.span()),
            Expression::Error(span) => span.clone(),
        }
    }
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Constant(constant) => write!(f, "{}", constant.inner),
            Expression::Variable(variable) => {
                let variable = variable.lock().unwrap();
                let show_id =
                    std::env::var("ORCO_SHOW_VAR_ID").map_or(false, |show_id| show_id == "1");
                if show_id {
                    write!(f, "{} (#{})", variable.name.inner, variable.id)
                } else {
                    write!(f, "{}", variable.name.inner)
                }
            }
            Expression::BinaryOp(lhs, op, rhs) => write!(f, "({} {} {})", lhs, op, rhs),
            Expression::UnaryOp(op, expr) => write!(f, "{}{}", op.inner, expr),
            Expression::Block(block) => write!(f, "{}", block.inner),
            Expression::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                write!(f, "if {} {}", condition, then_branch)?;
                if let Some(else_branch) = else_branch {
                    write!(f, " else {}", else_branch)?;
                }
                Ok(())
            }
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
            Expression::Assignment(target, value) => write!(f, "{} = {}", target, value),
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
    /// Equality
    Eq,
    /// Inequality
    Ne,
    /// Less than
    Lt,
    /// Less than or equal
    Le,
    /// Greater than
    Gt,
    /// Greater than or equal
    Ge,
}

impl std::fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Sub => write!(f, "-"),
            BinaryOp::Mul => write!(f, "*"),
            BinaryOp::Div => write!(f, "/"),
            BinaryOp::Mod => write!(f, "%"),
            BinaryOp::Eq => write!(f, "=="),
            BinaryOp::Ne => write!(f, "!="),
            BinaryOp::Lt => write!(f, "<"),
            BinaryOp::Le => write!(f, "<="),
            BinaryOp::Gt => write!(f, ">"),
            BinaryOp::Ge => write!(f, ">="),
        }
    }
}

/// Unary operators
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UnaryOp {
    /// Negation
    Neg,
}

impl std::fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOp::Neg => write!(f, "-"),
        }
    }
}
