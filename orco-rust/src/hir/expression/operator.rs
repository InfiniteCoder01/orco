use super::Expression;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OperatorKind {
    Call,
}

#[derive(Clone, Debug)]
pub struct Operator {
    pub kind: OperatorKind,
    pub args: Vec<Expression>,
}

impl Operator {
    pub fn new(kind: OperatorKind, args: Vec<Expression>) -> Self {
        Self { kind, args }
    }

    pub fn call(call: &syn::ExprCall, path: &crate::hir::Path) -> Self {
        Self {
            kind: OperatorKind::Call,
            args: std::iter::once(Expression::parse(&call.func, path))
                .chain(call.args.iter().map(|arg| Expression::parse(arg, path)))
                .collect(),
        }
    }

    pub fn build(
        &self,
        _builder: &mut crate::backend::FunctionBuilder<'_>,
    ) -> Vec<cranelift::prelude::Value> {
        // builder.0.ins().call(FN, args)
        todo!()
    }
}
