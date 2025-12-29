use crate::codegen;

/// Use this when a feature is not supported. Default implementation
pub struct Unsupported;

impl codegen::ACFCodegen for Unsupported {
    fn label(&mut self, _: codegen::Label) {
        unimplemented!("arbitrary control flow is not supported by this backend");
    }

    fn jump(&mut self, _: codegen::Label) {
        unimplemented!("arbitrary control flow is not supported by this backend");
    }

    fn cjump(&mut self, _: codegen::Operand, _: u128, _: bool, _: codegen::Label) {
        unimplemented!("arbitrary control flow is not supported by this backend");
    }
}
