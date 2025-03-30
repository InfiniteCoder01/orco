use super::{Context, Expression, ob};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OperatorKind {
    Call,
}

#[derive(Clone, Debug)]
pub struct Operator {
    pub kind: OperatorKind,
    pub args: Vec<Expression>,
    pub function: Option<ob::FunctionId>,
}

impl Operator {
    pub fn new(kind: OperatorKind, args: Vec<Expression>) -> Self {
        Self {
            kind,
            args,
            function: None,
        }
    }

    pub fn call(call: &syn::ExprCall, path: &crate::hir::Path) -> Self {
        Self::new(
            OperatorKind::Call,
            std::iter::once(Expression::parse(&call.func, path))
                .chain(call.args.iter().map(|arg| Expression::parse(arg, path)))
                .collect(),
        )
    }

    pub fn resolve(&mut self, ctx: &Context) {
        for arg in &mut self.args {
            arg.resolve(ctx);
        }
        match self.kind {
            OperatorKind::Call => {
                if let Expression::Symbol(super::Symbol::Function(id)) = &self.args[0] {
                    self.function = Some(*id);
                    self.args.remove(0);
                    return;
                }
            }
        }
    }

    pub fn build(&self, builder: &mut dyn ob::FunctionBuilder) -> ob::SSAValue {
        let args = self
            .args
            .iter()
            .map(|arg| arg.build(builder))
            .collect::<Vec<_>>();
        builder.call(self.function.unwrap(), &args)
    }
}
