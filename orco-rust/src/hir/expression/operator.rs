use super::{Context, Expression};
use crate::backend::cl::InstBuilder;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OperatorKind {
    Call,
}

#[derive(Clone, Debug)]
pub struct Operator {
    kind: OperatorKind,
    args: Vec<Expression>,
}

impl Operator {
    pub fn new(kind: OperatorKind, args: Vec<Expression>) -> Self {
        Self { kind, args }
    }

    pub fn call(ctx: &mut Context, call: &syn::ExprCall) -> Self {
        Self {
            kind: OperatorKind::Call,
            args: std::iter::once(Expression::parse(ctx, &call.func))
                .chain(call.args.iter().map(|arg| Expression::parse(ctx, arg)))
                .collect(),
        }
    }

    pub fn build(
        &self,
        builder: &mut crate::backend::FunctionBuilder<'_>,
    ) -> Vec<cranelift::prelude::Value> {
        // builder.0.ins().call(FN, args)
        todo!()
    }
}
