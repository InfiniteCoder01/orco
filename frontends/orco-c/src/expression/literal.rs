use super::*;

#[derive(Clone, PartialEq, Eq, Parse, ToTokens)]
pub enum Literal {
    Integer(LitUint),
}
impl Literal {
    pub fn build(&self, ctx: &mut orco::TypeInferenceContext) -> orco::expression::Literal {
        match self {
            Self::Integer(literal) => {
                orco::expression::Literal::Integer(literal.value() as _, orco::Type::Wildcard)
            }
        }
    }
}
