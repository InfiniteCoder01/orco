pub trait SignatureBuilder {
    fn finish(self: Box<Self>);
}

pub struct SSAValue(pub usize);
pub trait FunctionBuilder {
    fn unit(&mut self) -> SSAValue;
    fn i32(&mut self, value: i32) -> SSAValue;
    fn ret(&mut self, value: SSAValue);

    fn finish(self: Box<Self>);
}
