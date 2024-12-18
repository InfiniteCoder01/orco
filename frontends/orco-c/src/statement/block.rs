use super::*;

#[derive(Clone, PartialEq, Eq, Parse, ToTokens)]
pub struct Block(#[parsel(recursive)] pub Brace<Many<Statement>>);

impl orco::expression::Block for Block {
    fn expressions(&self) -> impl Iterator<Item = orco::Expression> {
        Box::new(self.0.iter().flat_map(|statement| statement.as_orco()))
    }

    fn expressions_mut(&mut self) -> impl Iterator<Item = orco::Expression<orco::Mut>> {
        Box::new(
            self.0
                .iter_mut()
                .flat_map(|statement| statement.as_orco_mut()),
        )
    }
}
