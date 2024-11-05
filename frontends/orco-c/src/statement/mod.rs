use super::*;
use parsel::ast::Brace;

#[derive(Clone, PartialEq, Eq, Parse, ToTokens)]
pub enum Statement {
    Block(Block),
    Expression(Expression),
    Empty,
}

#[derive(Clone, PartialEq, Eq, Parse, ToTokens)]
pub struct Block(#[parsel(recursive)] pub Brace<Many<Statement>>);
