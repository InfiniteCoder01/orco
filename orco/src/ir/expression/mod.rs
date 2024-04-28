use crate::diagnostics::*;
use crate::{ir::Type, TypeInference};

/// Constant value
pub mod constant;
pub use constant::Constant;

/// Code block
pub mod block;
pub use block::Block;

/// An expression
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Expression {
    /// A constant value
    Constant(Spanned<Constant>),
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
    VariableDeclaration {
        /// Variable name
        name: Spanned<String>,
        /// Is variable mutable?
        mutable: Spanned<bool>,
        /// Variable type
        r#type: Spanned<Type>,
        /// Initial value (optional (I wish it was nesessarry))
        value: Option<Box<Expression>>,
        /// Span of the whole expression
        span: Span,
    },
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
            Expression::VariableDeclaration { r#type, value, .. } => {
                if let Some(value) = value {
                    let value_type = value.infer_types(r#type, type_inference);
                    r#type.inner = type_inference.complete(r#type.inner.clone());
                    type_inference.equate(r#type, &value_type);
                } else {
                    r#type.inner = type_inference.complete(r#type.inner.clone());
                }
                Type::Unit
            }
            Expression::Error(_) => Type::Error,
        };
        r#type
    }

    /// Finish types and check them
    pub fn finish_and_check_types(&mut self, type_inference: &mut TypeInference) -> Type {
        let r#type = match self {
            Expression::Constant(constant) => constant.inner.finish_and_check_types(type_inference),
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
                        let mut colors = ColorGenerator::new();
                        let report = Report::build(
                            ReportKind::Error,
                            args.span.0.clone(),
                            args.span.1.start,
                        )
                        .with_message(format!(
                            "Argument count mismatch: Function '{}' expects {} arguments, but {} were given",
                            name.inner,
                            signature.args.len(),
                            args.inner.len()
                        ))
                        .with_label(
                            Label::new(args.span.clone())
                                .with_message("Here")
                                .with_color(colors.next()),
                        );
                        type_inference.reporter.report(report.finish());
                    }
                    for (arg, signature_arg) in std::iter::zip(&mut args.inner, &signature.args) {
                        let arg_type = arg.finish_and_check_types(type_inference);
                        if !arg_type.morphs(&signature_arg.1) {
                            let mut colors = ColorGenerator::new();
                            let report = Report::build(
                                ReportKind::Error,
                                arg.span().0.clone(),
                                arg.span().1.start,
                            )
                            .with_message(format!(
                                "Incompatible argument types: expected '{}', got '{}'",
                                arg_type, signature_arg.1.inner
                            ))
                            .with_label(
                                Label::new(arg.span().clone())
                                    .with_message("Here")
                                    .with_color(colors.next()),
                            )
                            .with_label(
                                Label::new(signature_arg.1.span.clone())
                                    .with_message("Expected becase of this")
                                    .with_color(colors.next()),
                            );
                            type_inference.reporter.report(report.finish());
                        }
                    }
                    (*signature.return_type).clone()
                } else {
                    let span = name.span.extend(&args.span);
                    let mut colors = ColorGenerator::new();
                    let report = Report::build(ReportKind::Error, span.0.clone(), span.1.start)
                        .with_message(format!(
                            "Function '{}' was not found in this scope",
                            name.inner
                        ))
                        .with_label(
                            Label::new(span)
                                .with_message("Here")
                                .with_color(colors.next()),
                        );
                    type_inference.reporter.report(report.finish());
                    Type::Error
                }
            }
            Expression::Return(expr) => {
                let r#type = expr.finish_and_check_types(type_inference);
                if !r#type.morphs(&type_inference.return_type) {
                    let expr_span = expr.span();
                    let mut colors = ColorGenerator::new();
                    let report = Report::build(
                        ReportKind::Error,
                        self.span().0.clone(),
                        self.span().1.start,
                    )
                    .with_message(format!(
                        "Return type mismatch: expected {}, got {}",
                        type_inference.return_type.inner, r#type
                    ))
                    .with_label(
                        Label::new(expr_span)
                            .with_message("Here")
                            .with_color(colors.next()),
                    )
                    .with_label(
                        Label::new(type_inference.return_type.span.clone())
                            .with_message("Expected becase of this")
                            .with_color(colors.next()),
                    );
                    type_inference.reporter.report(report.finish());
                }
                Type::Never
            }
            Expression::VariableDeclaration { r#type, value, .. } => {
                if let Some(value) = value {
                    let value_type = value.finish_and_check_types(type_inference);
                    type_inference.finish(r#type);
                    if !value_type.morphs(r#type) {
                        let r#type = r#type.clone();
                        let value_span = value.span();
                        let mut colors = ColorGenerator::new();
                        let report = Report::build(
                            ReportKind::Error,
                            self.span().0.clone(),
                            self.span().1.start,
                        )
                        .with_message(format!(
                            "Type mismatch: Expected '{}', got '{}'",
                            r#type.inner, value_type
                        ))
                        .with_label(
                            Label::new(value_span)
                                .with_message("This expression has an invalid type")
                                .with_color(colors.next()),
                        )
                        .with_label(
                            Label::new(r#type.span)
                                .with_message("Expected becase of this")
                                .with_color(colors.next()),
                        );
                        type_inference.reporter.report(report.finish());
                    }
                } else {
                    r#type.inner = type_inference.complete(r#type.inner.clone());
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
            Expression::BinaryOp(lhs, _, rhs) => lhs.span().extend(&rhs.span()),
            Expression::Block(block) => block.span.clone(),
            Expression::FunctionCall { name, args } => name.span.extend(&args.span),
            Expression::Return(expr) => expr.span.clone(),
            Expression::VariableDeclaration { span, .. } => span.clone(),
            Expression::Error(span) => span.clone(),
        }
    }
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Constant(constant) => write!(f, "{}", constant.inner),
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
            Expression::VariableDeclaration {
                name,
                mutable,
                r#type,
                value,
                ..
            } => {
                write!(f, "let ")?;
                if **mutable {
                    write!(f, "mut ")?;
                }
                write!(f, "{}: {}", name.inner, r#type.inner)?;
                if let Some(value) = value {
                    write!(f, " = {}", value)?;
                }
                Ok(())
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
