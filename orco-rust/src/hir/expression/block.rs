use super::{Context, Expression};
use syn::spanned::Spanned as _;

#[derive(Clone, Debug)]
pub struct Block {
    pub span: proc_macro2::Span,
    pub statements: Vec<Expression>,
    pub tail: Option<Box<Expression>>,
}

impl Block {
    pub fn parse(ctx: &mut Context, value: &syn::Block) -> Self {
        let mut block = Self {
            span: value.span(),
            statements: Vec::with_capacity(value.stmts.len()),
            tail: None,
        };
        for (idx, stmt) in value.stmts.iter().enumerate() {
            match stmt {
                syn::Stmt::Local(local) => todo!(),
                syn::Stmt::Item(item) => todo!(),
                syn::Stmt::Expr(expr, semi) => {
                    if idx + 1 == value.stmts.len() && semi.is_none() {
                        block.tail = Some(Box::new(Expression::parse(ctx, expr)));
                    } else {
                        block.statements.push(Expression::parse(ctx, expr));
                    }
                }
                syn::Stmt::Macro(stmt_macro) => todo!(),
            }
        }
        block
    }
}
