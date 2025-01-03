use super::*;

#[derive(Parse, ToTokens)]
pub struct Block(#[parsel(recursive)] pub Brace<Many<Statement>>);

impl orco::expression::Block for Block {
    fn expressions(&self) -> orco::DynIter<orco::Expression> {
        Box::new(self.0.iter().map(Statement::as_orco))
    }

    fn expressions_mut(&mut self) -> orco::DynIter<orco::Expression<orco::Mut>> {
        Box::new(self.0.iter_mut().map(Statement::as_orco_mut))
    }
}
