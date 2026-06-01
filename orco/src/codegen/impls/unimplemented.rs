use crate::codegen as cg;

/// Use this when a feature is not supported. Default implementation
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Unimplemented;

impl cg::Intrinsics for Unimplemented {}

impl cg::AcfCodegen for Unimplemented {
    fn alloc_label(&mut self) -> cg::Label {
        unimplemented!("arbitrary control flow is not supported by this backend");
    }

    fn label(&mut self, _: cg::Label) {
        unimplemented!("arbitrary control flow is not supported by this backend");
    }

    fn jump(&mut self, _: cg::Label) {
        unimplemented!("arbitrary control flow is not supported by this backend");
    }

    fn cjump(&mut self, _: cg::Value, _: cg::Label) {
        unimplemented!("arbitrary control flow is not supported by this backend");
    }
}

impl cg::BcfCodegen for Unimplemented {
    fn if_(&mut self, _: cg::Value) {
        unimplemented!("regular control flow is not supported by this backend");
    }

    fn else_(&mut self) {
        unimplemented!("regular control flow is not supported by this backend");
    }

    fn end(&mut self) {
        unimplemented!("regular control flow is not supported by this backend");
    }

    fn loop_(&mut self) {
        unimplemented!("regular control flow is not supported by this backend");
    }

    fn break_(&mut self) {
        unimplemented!("regular control flow is not supported by this backend");
    }

    fn continue_(&mut self) {
        unimplemented!("regular control flow is not supported by this backend");
    }
}
