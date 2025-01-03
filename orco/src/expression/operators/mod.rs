use super::*;

/// This expression should be interpreted as an operator
pub trait AsOperator {
    /// Infer the types, bind this reference to an actual operator
    fn infer_types(&mut self, ctx: &mut TypeInferenceContext);
    /// Returns the operator
    fn operator(&self) -> Option<&dyn Operator>;
}

/// Operator trait. Every operator implements this trait
pub trait Operator: std::fmt::Debug + std::fmt::Display {}

#[debug_display]
impl std::fmt::Display for &dyn AsOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.operator() {
            Some(operator) => std::fmt::Display::fmt(operator, f),
            None => write!(f, "<Unknown operator>"),
        }
    }
}
