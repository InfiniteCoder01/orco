use super::*;

pub trait Operator {}
pub trait OperatorReference {}

pub trait OperatorCall {
    fn operator(&self) -> SymbolRef<dyn Operator, dyn OperatorReference>;
    fn args(&self) -> Vec<Expression>;
    fn args_mut(&self) -> Vec<Expression<Mut>>;
}

#[debug_display]
impl std::fmt::Display for &dyn OperatorCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
