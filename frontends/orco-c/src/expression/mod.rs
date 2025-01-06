use super::*;
use parsel::ast::LitUint;

// pub mod functions;
// pub use functions::FunctionCall;
pub mod literal;
pub use literal::Literal;

#[derive(Clone, PartialEq, Eq, Parse, ToTokens)]
pub enum Expression {
    Literal(Literal),
}

impl Expression {
    pub fn build(
        &self,
        ctx: &mut orco::TypeInferenceContext,
        _expressions: &mut Vec<orco::Expression>,
    ) -> orco::Expression {
        match self {
            Expression::Literal(literal) => orco::Expression::Literal(literal.build(ctx)),
        }
    }
}
