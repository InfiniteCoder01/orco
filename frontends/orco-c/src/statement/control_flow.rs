use super::*;

#[derive(Clone, PartialEq, Eq, Parse, ToTokens)]
pub struct Return {
    pub kw_return: kw::Return,
    pub expression: Expression,
    pub op_semi: Semi,
}

impl orco::expression::Return for Return {
    fn expression(&self) -> orco::Expression {
        self.expression.as_orco()
    }

    fn expression_mut(&mut self) -> orco::Expression<orco::Mut> {
        self.expression.as_orco_mut()
    }
}
