use super::*;

#[derive(Clone, PartialEq, Eq, Parse, ToTokens)]
pub struct IntegerLiteral(pub LitUint);

impl orco::expression::literal::IntegerLiteral for IntegerLiteral {
    fn r#type(&self) -> orco::Type {
        orco::Type::Integer(32)
    }

    fn value(&self) -> u128 {
        self.0.value() as _
    }
}
