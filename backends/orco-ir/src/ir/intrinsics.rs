use super::Expression;

/// Intrinsic function calls, see [`oc::Intrinsics`]
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Intrinsic {
    /// See [`oc::Intrinsics::add`]
    Add(Box<Expression>, Box<Expression>),
    /// See [`oc::Intrinsics::mul`]
    Mul(Box<Expression>, Box<Expression>),
}

impl Intrinsic {
    /// Weather this intrinsic produces a return value.
    /// Similar to [super::Statement::is_expression]
    pub fn is_expression(&self) -> bool {
        true
    }

    /// Get type of the value this intrinsic produces.
    /// Similar to [super::Statement::get_type]
    pub fn get_type(&self, backend: &crate::Backend, body: &super::Body) -> orco::Type {
        match self {
            Self::Add(a, _) => a.get_type(backend, body),
            Self::Mul(a, _) => a.get_type(backend, body),
        }
    }
}

impl std::fmt::Display for Intrinsic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Intrinsic::Add(a, b) => write!(f, "{a} + {b}"),
            Intrinsic::Mul(a, b) => write!(f, "{a} * {b}"),
        }
    }
}
