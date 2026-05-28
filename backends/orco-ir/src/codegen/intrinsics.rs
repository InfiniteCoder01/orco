use super::{ir, oc};

impl oc::Intrinsics for &mut super::Codegen<'_, '_> {
    fn add(&mut self, a: oc::Value, b: oc::Value) -> oc::Value {
        let a = Box::new(self.use_value(a));
        let b = Box::new(self.use_value(b));
        self.expr(ir::Expression::Intrinsic(ir::Intrinsic::Add(a, b)))
    }

    fn mul(&mut self, a: oc::Value, b: oc::Value) -> oc::Value {
        let a = Box::new(self.use_value(a));
        let b = Box::new(self.use_value(b));
        self.expr(ir::Expression::Intrinsic(ir::Intrinsic::Mul(a, b)))
    }
}
