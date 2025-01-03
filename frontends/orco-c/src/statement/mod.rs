use super::*;
use parsel::{ast::Brace, syn::token::Semi};

pub mod block;
pub use block::Block;
pub mod control_flow;
pub use control_flow::{Return, ReturnExpression};
pub mod variables;
pub use variables::VariableDeclaration;

#[derive(Parse, ToTokens)]
pub enum Statement {
    Block(Block),
    Return(ReturnExpression),
    VariableDeclaration(VariableDeclaration),
    Expression(Expression, Semi),
    Empty(Semi),
}

impl Statement {
    pub fn as_orco(&self) -> orco::Expression {
        match self {
            Self::Block(block) => orco::Expression::Block(block as _),
            Self::Return(expr) => orco::Expression::Operator(&expr.0 as _),
            Self::VariableDeclaration(decl) => orco::Expression::VariableDeclaration(decl as _),
            Self::Expression(expr, _) => expr.as_orco(),
            Self::Empty(_) => todo!(),
        }
    }

    pub fn as_orco_mut(&mut self) -> orco::Expression<orco::Mut> {
        match self {
            Self::Block(block) => orco::Expression::Block(block as _),
            Self::Return(expr) => orco::Expression::Operator(&mut expr.0 as _),
            Self::VariableDeclaration(decl) => orco::Expression::VariableDeclaration(decl as _),
            Self::Expression(expr, _) => expr.as_orco_mut(),
            Self::Empty(_) => todo!(),
        }
    }
}
