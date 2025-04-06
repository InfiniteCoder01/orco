use super::{Context, ob};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Symbol {
    Unresolved(crate::hir::Path),
    Function(ob::FunctionId),
}

impl Symbol {
    pub fn parse(path: &syn::Path) -> Self {
        Self::Unresolved(path.into())
    }

    pub fn resolve(&mut self, ctx: &Context) {
        match self {
            Self::Unresolved(path) => {
                let Some(symbol) = ctx.registry.get_symbol(&path.to_string()) else {
                    ctx.diag
                        .symbol_not_found(format!("symbol '{path}'"))
                        .mark(path.span());
                    return;
                };
                match symbol {
                    orco::Symbol::Function(id) => {
                        *self = Self::Function(id);
                        self.resolve(ctx);
                    }
                }
            }
            Self::Function(_) => (),
        }
    }

    pub fn build(&self, _builder: &mut dyn ob::FunctionBuilder) -> ob::SSAValue {
        todo!()
    }
}
