use super::*;
use parsel::{ast::Brace, syn::token::Semi};

pub mod block;
pub use block::Block;
pub mod control_flow;
pub use control_flow::Return;
pub mod variables;
pub use variables::VariableDeclaration;

#[derive(Parse, ToTokens)]
pub enum Statement {
    Block(Block),
    Return(Return),
    VariableDeclaration(VariableDeclaration),
    Expression(Expression, Semi),
    Empty(Semi),
}

impl Statement {
    pub fn build(
        &self,
        ctx: &mut orco::TypeInferenceContext,
        expressions: &mut Vec<orco::Expression>,
    ) {
        match self {
            Statement::Block(block) => block.build(ctx, expressions),
            Statement::Return(r#return) => r#return.build(ctx, expressions),
            Statement::VariableDeclaration(_) => todo!(),
            Statement::Expression(expression, _) => expression.build(ctx, expressions),
            Statement::Empty(_) => (),
        }
    }
}
