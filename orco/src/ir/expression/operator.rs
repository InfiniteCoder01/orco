use super::*;

/// Binary expression
#[derive(Clone, Debug)]
pub struct BinaryExpression {
    /// Left hand side
    pub lhs: Box<Expression>,
    /// Operator
    pub op: BinaryOp,
    /// Right hand side
    pub rhs: Box<Expression>,
}

impl BinaryExpression {
    /// Create a new binary expression
    pub fn new(lhs: Box<Expression>, op: BinaryOp, rhs: Box<Expression>) -> Self {
        Self { lhs, op, rhs }
    }

    /// Get the type this binary expression evaluates to
    pub fn get_type(&self) -> Type {
        match self.op {
            BinaryOp::Eq
            | BinaryOp::Ne
            | BinaryOp::Lt
            | BinaryOp::Le
            | BinaryOp::Gt
            | BinaryOp::Ge => Type::Bool,
            _ => self.lhs.get_type() | self.rhs.get_type(),
        }
    }

    /// Infer types
    pub fn infer_types(&mut self, type_inference: &mut TypeInference) -> Type {
        match self.op {
            BinaryOp::Eq
            | BinaryOp::Ne
            | BinaryOp::Lt
            | BinaryOp::Le
            | BinaryOp::Gt
            | BinaryOp::Ge => {
                let lhs_type = self.lhs.infer_types(type_inference);
                let rhs_type = self.rhs.infer_types(type_inference);
                type_inference.equate(&lhs_type, &rhs_type);
                Type::Bool
            }
            _ => {
                let lhs_type = self.lhs.infer_types(type_inference);
                let rhs_type = self.rhs.infer_types(type_inference);
                type_inference.equate(&lhs_type, &rhs_type)
            }
        }
    }

    /// Finish and check types
    pub fn finish_and_check_types(&mut self, type_inference: &mut TypeInference) -> Type {
        let lhs_type = self.lhs.finish_and_check_types(type_inference);
        let rhs_type = self.rhs.finish_and_check_types(type_inference);
        if !rhs_type.morphs(&lhs_type) {
            let mut colors = ColorGenerator::new();
            let report = Report::build(
                ReportKind::Error,
                self.lhs.span().0.clone(),
                self.lhs.span().1.start,
            )
            .with_message(format!(
                "Incompatible types for binary operation '{}': '{}' and '{}'",
                self.op, lhs_type, rhs_type
            ))
            .with_label(
                Label::new(self.lhs.span().clone())
                    .with_message("Left hand side")
                    .with_color(colors.next()),
            )
            .with_label(
                Label::new(self.rhs.span().clone())
                    .with_message("Right hand side")
                    .with_color(colors.next()),
            );
            type_inference.reporter.report_ariadne(report.finish());
            todo!(
                "Type mismatch for binary operator error: {:?} and {:?}",
                lhs_type,
                rhs_type
            );
        }
        match self.op {
            BinaryOp::Eq
            | BinaryOp::Ne
            | BinaryOp::Lt
            | BinaryOp::Le
            | BinaryOp::Gt
            | BinaryOp::Ge => Type::Bool,
            _ => lhs_type | rhs_type,
        }
    }
}

impl std::fmt::Display for BinaryExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {} {})", self.lhs, self.op, self.rhs)
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

/// Unary expression
#[derive(Clone, Debug)]
pub struct UnaryExpression {
    /// Operator
    pub op: UnaryOp,
    /// Expression
    pub expr: Box<Expression>,
}

impl UnaryExpression {
    /// Create a new unary expression
    pub fn new(op: UnaryOp, expr: Box<Expression>) -> Self {
        Self { op, expr }
    }

    /// Get the type this unary expression evaluates to
    pub fn get_type(&self) -> Type {
        self.expr.get_type()
    }

    /// Infer types
    pub fn infer_types(&mut self, type_inference: &mut TypeInference) -> Type {
        self.expr.infer_types(type_inference)
    }

    /// Finish and check types
    pub fn finish_and_check_types(&mut self, type_inference: &mut TypeInference) -> Type {
        let r#type = self.expr.finish_and_check_types(type_inference);
        match self.op {
            UnaryOp::Neg => match r#type {
                Type::Int(_) | Type::Float(_) => (),
                _ => {
                    type_inference.reporter.report_type_error(
                        format!("Cannot apply unary negation to {}", r#type),
                        self.expr.span(),
                        vec![],
                    );
                }
            },
        }
        r#type
    }
}

impl std::fmt::Display for UnaryExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.op, self.expr)
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

/// Assignment expression
#[derive(Clone, Debug)]
pub struct AssignmentExpression {
    /// Target
    pub target: Box<Expression>,
    /// Value
    pub value: Box<Expression>,
}

impl AssignmentExpression {
    /// Create a new assignment expression
    pub fn new(target: Box<Expression>, value: Box<Expression>) -> Self {
        Self { target, value }
    }

    /// Infer types for this assignment expression
    pub fn infer_types(&mut self, type_inference: &mut TypeInference) -> Type {
        let value = self.value.infer_types(type_inference);
        let target = self.target.infer_types(type_inference);
        type_inference.equate(&target, &value);
        Type::Unit
    }

    /// Finish and check types
    pub fn finish_and_check_types(&mut self, type_inference: &mut TypeInference) -> Type {
        let value_type = self.value.finish_and_check_types(type_inference);
        let target_type = self.target.finish_and_check_types(type_inference);
        let can_assign = match self.target.as_ref() {
            Expression::Symbol(symbol) => {
                if let SymbolReference::Variable(variable) = &symbol.inner {
                    if !variable.mutable.inner {
                        type_inference.reporter.report_type_error(
                            format!("Cannot assign to an immutable variable '{}'", variable.name),
                            self.target.span(),
                            vec![(
                                "Help: Make this variable mutable",
                                variable.mutable.span.clone(),
                            )],
                        );
                    }
                    true
                } else {
                    false
                }
            }
            _ => false,
        };
        if !can_assign {
            type_inference.reporter.report_type_error(
                format!("Cannot assign to '{}'", self.target),
                self.target.span(),
                vec![],
            );
        };
        if !value_type.morphs(&target_type) {
            type_inference.reporter.report_type_error(
                format!("Cannot assign '{}' to '{}'", value_type, target_type),
                self.value.span(),
                vec![("Expected because of this", self.target.span())],
            );
        }
        Type::Unit
    }
}

impl std::fmt::Display for AssignmentExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = {}", self.target, self.value)
    }
}
