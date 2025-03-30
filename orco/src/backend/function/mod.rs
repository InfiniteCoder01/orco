use super::FunctionId;

pub trait SignatureBuilder {
    fn public(&mut self);
    fn external(&mut self);
    fn finish(self: Box<Self>);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SSAValue(pub usize);

pub trait FunctionBuilder {
    fn unit(&mut self) -> SSAValue;
    fn i32(&mut self, value: i32) -> SSAValue;
    fn call(&mut self, id: FunctionId, args: &[SSAValue]) -> SSAValue;
    fn ret(&mut self, value: SSAValue);

    fn finish(self: Box<Self>);
}
