use super::Expression;
use syn::spanned::Spanned as _;

#[derive(Clone, Debug)]
pub struct Block {
    pub span: miette::SourceSpan,
    pub statements: Vec<Expression>,
    pub tail: Option<Box<Expression>>,
}

impl Block {
    pub fn parse(value: &syn::Block, path: &crate::hir::Path) -> Self {
        let mut block = Self {
            span: value.span().byte_range().into(),
            statements: Vec::with_capacity(value.stmts.len()),
            tail: None,
        };
        for (idx, stmt) in value.stmts.iter().enumerate() {
            match stmt {
                syn::Stmt::Local(_) => todo!(),
                syn::Stmt::Item(_) => todo!(),
                syn::Stmt::Expr(expr, semi) => {
                    if idx + 1 == value.stmts.len() && semi.is_none() {
                        block.tail = Some(Box::new(Expression::parse(expr, path)));
                    } else {
                        block.statements.push(Expression::parse(expr, path));
                    }
                }
                syn::Stmt::Macro(_) => todo!(),
            }
        }
        block
    }

    pub fn build(
        &self,
        builder: &mut crate::backend::FunctionBuilder<'_>,
    ) -> Vec<cranelift::prelude::Value> {
        for statement in &self.statements {
            statement.build(builder);
        }
        self.tail
            .as_ref()
            .map_or_else(Vec::new, |expr| expr.build(builder))
    }
}
