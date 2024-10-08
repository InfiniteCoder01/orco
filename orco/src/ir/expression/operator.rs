use super::*;

/// Binary expression
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct BinaryExpression {
    /// Left hand side
    pub lhs: Box<Expression>,
    /// Operator
    pub op: BinaryOp,
    /// Right hand side
    pub rhs: Box<Expression>,
    /// Span of the expression
    #[derivative(Debug = "ignore")]
    pub span: Option<Span>,
    /// Metadata
    #[derivative(Debug = "ignore")]
    pub metadata: Box<dyn BinaryMetadata>,
}

impl BinaryExpression {
    /// Create a new binary expression
    pub fn new(
        lhs: Box<Expression>,
        op: BinaryOp,
        rhs: Box<Expression>,
        span: Option<Span>,
        metadata: impl BinaryMetadata + 'static,
    ) -> Self {
        Self {
            lhs,
            op,
            rhs,
            span,
            metadata: Box::new(metadata),
        }
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
            _ => self.lhs.get_type() | &self.rhs.get_type(),
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
                type_inference.equate(&lhs_type, &rhs_type);
                lhs_type
            }
        }
    }

    /// Finish and check types
    pub fn finish_and_check_types(&mut self, type_inference: &mut TypeInference) -> Type {
        let lhs_type = self.lhs.finish_and_check_types(type_inference);
        let rhs_type = self.rhs.finish_and_check_types(type_inference);
        if !rhs_type.morphs(&lhs_type) {
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
            _ => lhs_type | &rhs_type,
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

/// Frontend metadata for binary expression
pub trait BinaryMetadata: Metadata {}
impl_metadata!(BinaryMetadata);

/// Unary expression
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct UnaryExpression {
    /// Operator
    pub op: UnaryOp,
    /// Expression
    pub expr: Box<Expression>,
    /// Span of the expression
    #[derivative(Debug = "ignore")]
    pub span: Option<Span>,
    /// Metadata
    #[derivative(Debug = "ignore")]
    pub metadata: Box<dyn UnaryMetadata>,
}

impl UnaryExpression {
    /// Create a new unary expression
    pub fn new(
        op: UnaryOp,
        expr: Box<Expression>,
        span: Option<Span>,
        metadata: impl UnaryMetadata + 'static,
    ) -> Self {
        Self {
            op,
            expr,
            span,
            metadata: Box::new(metadata),
        }
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
                _ => todo!("Type mismatch on unary operator error: {}", r#type),
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

/// Frontend metadata for unary expression
pub trait UnaryMetadata: Metadata {}
impl_metadata!(UnaryMetadata);

/// Assignment expression
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct AssignmentExpression {
    /// Target
    pub target: Box<Expression>,
    /// Value
    pub value: Box<Expression>,
    /// Span of the expression
    #[derivative(Debug = "ignore")]
    pub span: Option<Span>,
    /// Metadata
    #[derivative(Debug = "ignore")]
    pub metadata: Box<dyn AssignmentMetadata>,
}

impl AssignmentExpression {
    /// Create a new assignment expression
    pub fn new(
        target: Box<Expression>,
        value: Box<Expression>,
        span: Option<Span>,
        metadata: impl AssignmentMetadata + 'static,
    ) -> Self {
        Self {
            target,
            value,
            span,
            metadata: Box::new(metadata),
        }
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
        let can_assign = true;
        match self.target.as_ref() {
            Expression::Symbol(symbol, ..) => {
                if let SymbolReference::Variable(variable) = symbol.inner {
                    if !variable.mutable.inner {
                        todo!(
                            "Cannot assign to an immutable variable error: '{}'",
                            variable.name
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
            todo!("Cannot assign to error: '{}'", self.target);
        }
        if !value_type.morphs(&target_type) {
            todo!("Cannot assign error: '{}' to '{}'", value_type, target_type);
        }
        Type::Unit
    }
}

impl std::fmt::Display for AssignmentExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = {}", self.target, self.value)
    }
}

/// Frontend metadata for assignment expression
pub trait AssignmentMetadata: Metadata {}
impl_metadata!(AssignmentMetadata);
