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
    pub fn as_orco(&self) -> orco::Expression {
        match self {
            Self::Block(block) => block.statements(),
            Self::Return(expr) => expr.as_orco(),
            Self::VariableDeclaration(decl) => {
                std::iter::once(orco::Expression::VariableDeclaration(decl as _))
            }
            Self::Expression(expr, _) => expr.as_orco(),
            Self::Empty(_) => std::iter::empty(),
        }
    }
}
