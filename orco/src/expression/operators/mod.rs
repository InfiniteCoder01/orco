use super::*;

pub trait Operator {}

pub trait OperatorHandler {
    fn args(&self) -> Vec<Expression>;
    fn args_mut(&mut self) -> Vec<Expression<Mut>>;
}

pub trait OperatorCall {
    fn object(&self) -> Option<symbol_box::Guard<dyn Operator>>;
    fn handler(&self) -> &std::sync::RwLock<dyn OperatorHandler>;
}

impl<H: OperatorHandler + 'static> OperatorCall for SymbolRef<dyn Operator, H> {
    fn object(&self) -> Option<symbol_box::Guard<dyn Operator>> {
        SymbolRef::object(self) as _
    }

    fn handler(&self) -> &std::sync::RwLock<dyn OperatorHandler> {
        SymbolRef::handler(self) as _
    }
}

#[debug_display]
impl std::fmt::Display for &dyn OperatorCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OPERATOR ")?;
        for (i, arg) in self.handler().read().unwrap().args().iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", arg)?;
        }
        Ok(())
    }
}
