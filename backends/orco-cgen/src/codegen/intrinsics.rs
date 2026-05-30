use super::{ValueInfo, oc};

impl oc::Intrinsics for &mut super::Codegen<'_, '_> {
    fn add(&mut self, a: oc::Value, b: oc::Value) -> oc::Value {
        let (a, b) = (self.use_value(a), self.use_value(b));
        assert_eq!(a.ty, b.ty, "can't add values of different types");
        self.mk_value(ValueInfo::new(
            format!("({} + {})", a.expression, b.expression),
            a.ty,
        ))
    }

    fn mul(&mut self, a: oc::Value, b: oc::Value) -> oc::Value {
        let (a, b) = (self.use_value(a), self.use_value(b));
        assert_eq!(a.ty, b.ty, "can't add values of different types");
        self.mk_value(ValueInfo::new(
            format!("({} * {})", a.expression, b.expression),
            a.ty,
        ))
    }

    fn eq(&mut self, a: oc::Value, b: oc::Value) -> oc::Value {
        let (a, b) = (self.use_value(a), self.use_value(b));
        assert_eq!(a.ty, b.ty, "can't eq values of different types");
        self.mk_value(ValueInfo::new(
            format!("({} == {})", a.expression, b.expression),
            a.ty,
        ))
    }
}
