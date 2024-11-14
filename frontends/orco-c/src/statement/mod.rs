use super::*;
use parsel::{ast::Brace, syn::token::Semi};

pub mod block;
pub use block::Block;
pub mod control_flow;
pub use control_flow::Return;
pub mod variables;
pub use variables::VariableDeclaration;

#[derive(Clone, PartialEq, Eq, Parse, ToTokens)]
pub enum Statement {
    Block(Block),
    Return(Return),
    VariableDeclaration(VariableDeclaration),
    Expression(Expression, Semi),
    Empty(Semi),
}

impl Statement {
    #[orco::make_mut]
    pub fn as_orco(&self) -> orco::Expression {
        match self {
            Self::Block(block) => orco::Expression::Block(block as _),
            Self::Return(expr) => orco::Expression::Return(expr as _),
            Self::VariableDeclaration(decl) => orco::Expression::VariableDeclaration(decl as _),
            Self::Expression(_, _) => todo!(),
            Self::Empty(_) => todo!(),
        }
    }
}
