use super::*;
use parsel::ast::LitUint;

#[derive(Clone, PartialEq, Eq, Parse, ToTokens)]
pub enum Expression {
    Integer(IntegerLiteral),
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

#[derive(Clone, PartialEq, Eq, Parse, ToTokens)]
pub struct IntegerLiteral(pub LitUint);

impl orco::expression::literal::IntegerLiteral for IntegerLiteral {
    fn value(&self) -> u128 {
        self.0.value() as _
    }
}
