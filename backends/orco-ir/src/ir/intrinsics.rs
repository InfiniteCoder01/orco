use orco::codegen as oc;

/// Intrinsic function calls, see [`oc::Intrinsics`]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Intrinsic {
    /// See [`oc::Intrinsics::add`]
    Add(oc::Value, oc::Value),
    /// See [`oc::Intrinsics::mul`]
    Mul(oc::Value, oc::Value),
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
            Self::Add(a, _) => body.type_of(a.0, backend),
            Self::Mul(a, _) => body.type_of(a.0, backend),
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
