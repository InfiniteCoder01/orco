use super::Value;

/// Interface providing intrinsic function implementations.
pub trait Intrinsics {
    /// Integer/float addition
    #[allow(unused_variables)]
    fn add(&mut self, a: Value, b: Value) -> Value {
        unimplemented!("add operation");
    }

    /// Integer/float multiplication
    #[allow(unused_variables)]
    fn mul(&mut self, a: Value, b: Value) -> Value {
        unimplemented!("mul operation");
    }

    /// Primitive type equality check
    #[allow(unused_variables)]
    fn eq(&mut self, a: Value, b: Value) -> Value {
        unimplemented!("eq operation");
    }

    /// Logical/Bitwise not
    #[allow(unused_variables)]
    fn not(&mut self, a: Value) -> Value {
        unimplemented!("not operation");
    }
}
