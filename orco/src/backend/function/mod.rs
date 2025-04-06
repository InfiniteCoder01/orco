use super::FunctionId;

/// Signature builder. Takes types from registery, no need to specify them.
/// Don't forget to call [`SignatureBuilder::finish`]!
pub trait SignatureBuilder {
    /// Mark function as public
    fn public(&mut self);
    /// Import function from another translation unit (outside of orco).
    /// External functions don't need to be built
    fn external(&mut self);
    /// Finish building signature and add the function to registery
    fn finish(self: Box<Self>);
}

/// Single SSA value ID. Can be of a compound type
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SSAValue(pub usize);

/// Function builder. This is used to generate instructions.
/// Don't forget to call [`SignatureBuilder::finish`]!
pub trait FunctionBuilder {
    /// Generate a unit value (aka `void` in C)
    fn unit(&mut self) -> SSAValue;
    /// Generate a 32-bit integer value
    fn i32(&mut self, value: i32) -> SSAValue;
    /// Call a function, specify function ID you get from registering the function.
    /// Also takes arguments and returns an SSA Value
    fn call(&mut self, id: FunctionId, args: &[SSAValue]) -> SSAValue;
    /// Return from the function
    fn ret(&mut self, value: SSAValue);
    /// Finish building a function
    fn finish(self: Box<Self>);
}
