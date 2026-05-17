use super::{ir, oc};

impl oc::Intrinsics for &mut super::Codegen<'_, '_> {
    fn add(&mut self, a: oc::Value, b: oc::Value) -> oc::Value {
        self.stmt(ir::Statement::Intrinsic(ir::Intrinsic::Add(a, b)))
    }

    fn mul(&mut self, a: oc::Value, b: oc::Value) -> oc::Value {
        self.stmt(ir::Statement::Intrinsic(ir::Intrinsic::Mul(a, b)))
    }
}
