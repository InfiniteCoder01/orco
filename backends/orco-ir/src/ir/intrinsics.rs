use super::Expression;

/// Intrinsic function calls, see [`oc::Intrinsics`]
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Intrinsic {
    /// See [`oc::Intrinsics::add`]
    Add(Box<Expression>, Box<Expression>),
    /// See [`oc::Intrinsics::mul`]
    Mul(Box<Expression>, Box<Expression>),
    /// See [`oc::Intrinsics::eq`]
    Eq(Box<Expression>, Box<Expression>),
    /// See [`oc::Intrinsics::not`]
    Not(Box<Expression>),
}

impl Intrinsic {
    /// Weather this intrinsic produces a return value.
    /// Similar to [`super::Statement::is_expression`]
    #[must_use]
    pub fn is_expression(&self) -> bool {
        true
    }

    /// Get type of the value this intrinsic produces.
    /// Similar to [`super::Statement::get_type`]
    pub fn get_type(&self, store: &crate::Store, body: &super::Body) -> orco::Type {
        match self {
            Self::Add(a, _) => a.get_type(store, body),
            Self::Mul(a, _) => a.get_type(store, body),
            Self::Eq(a, _) => a.get_type(store, body),
            Self::Not(a) => a.get_type(store, body),
        }
    }
}

impl std::fmt::Display for Intrinsic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Intrinsic::Add(a, b) => write!(f, "{a} + {b}"),
            Intrinsic::Mul(a, b) => write!(f, "{a} * {b}"),
            Intrinsic::Eq(a, b) => write!(f, "{a} == {b}"),
            Intrinsic::Not(a) => write!(f, "!{a}"),
        }
    }
}
