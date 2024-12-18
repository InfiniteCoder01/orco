use super::*;

#[derive(Parse, ToTokens)]
pub struct Return {
    pub kw_return: symbol_box::SymbolRef<dyn orco::operators::Operator, kw::Return>,
    pub expression: Expression,
    pub op_semi: Semi,
}

impl Return {
    pub fn as_orco(&self) -> orco::Expression {
        // self.expression.as_orco()
        todo!()
    }

    pub fn as_orco_mut(&mut self) -> orco::Expression<orco::Mut> {
        // self.expression.as_orco_mut()
        todo!()
    }
}
