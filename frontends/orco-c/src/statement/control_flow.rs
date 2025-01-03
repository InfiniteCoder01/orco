use super::*;

pub type ReturnExpression = symbol_box::SymbolRef<dyn orco::operators::Operator, Return>;

#[derive(Parse, ToTokens)]
pub struct Return {
    pub kw_return: kw::Return,
    pub expression: Expression,
    pub op_semi: Semi,
}

impl orco::operators::OperatorHandler for Return {
    fn args(&self) -> Vec<orco::Expression> {
        vec![self.expression.as_orco()]
    }

    fn args_mut(&mut self) -> Vec<orco::Expression<orco::Mut>> {
        vec![self.expression.as_orco_mut()]
    }
}
