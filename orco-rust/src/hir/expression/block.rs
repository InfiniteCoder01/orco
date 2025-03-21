use super::{Expression, ob};
use syn::spanned::Spanned as _;

#[derive(Clone, Debug)]
pub struct Block {
    pub span: orco::diagnostic::Span,
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

    pub fn build(&self, builder: &mut dyn ob::FunctionBuilder) -> ob::SSAValue {
        for statement in &self.statements {
            statement.build(builder);
        }
        match self.tail.as_ref() {
            Some(expr) => expr.build(builder),
            None => builder.unit(),
        }
    }
}
