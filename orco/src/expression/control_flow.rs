use super::*;

/// Return a value from a function
pub trait Return {
    /// Get the value
    fn expression(&self) -> Expression;
    /// Version of [`Return::expression`] that returns mutable reference
    fn expression_mut(&mut self) -> Expression<Mut>;
}

impl std::fmt::Display for &dyn Return {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "return {}", self.expression().to_string().trim())
    }
}
