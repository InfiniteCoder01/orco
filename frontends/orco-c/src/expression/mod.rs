use super::*;
use parsel::ast::LitUint;

pub mod functions;
// pub use functions::FunctionCall;
pub mod literal;
pub use literal::IntegerLiteral;

#[derive(Clone, PartialEq, Eq, Parse, ToTokens)]
pub enum Expression {
    Integer(IntegerLiteral),
    // FunctionCall(FunctionCall),
}

impl Expression {
    #[orco::make_mut]
    pub fn as_orco(&self) -> orco::Expression {
        use orco::expression::Literal;
        match self {
            Self::Integer(literal) => orco::Expression::Literal(Literal::Integer(literal as _)),
        }
    }
}
