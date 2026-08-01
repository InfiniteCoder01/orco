use super::Value;

/// A label ID. See [`AcfCodegen::label`]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Label(pub usize);

/// Arbitrary control flow instructions, such as jumps.
/// Warning: Not all codegens implement arbitrary control flow
pub trait AcfCodegen {
    /// Allocates a label to be placed later
    fn alloc_label(&mut self) -> Label {
        unimplemented!("arbitrary control flow is not supported by this backend")
    }

    /// Places a label in the current position.
    fn label(&mut self, label: Label) {
        unimplemented!("arbitrary control flow is not supported by this backend")
    }

    /// Jump to a label.
    /// See [`AcfCodegen::label`]
    fn jump(&mut self, label: Label) {
        unimplemented!("arbitrary control flow is not supported by this backend")
    }

    /// Jumps if condition is true.
    /// See [`AcfCodegen::label`]
    fn cjump(&mut self, condition: Value, label: Label) {
        unimplemented!("arbitrary control flow is not supported by this backend")
    }
}

/// Block control flow (somewhat traditional/wasm style).
pub trait BcfCodegen {
    /// Starts a block that will only be executed if the condition is met
    fn if_(&mut self, condition: Value) {
        todo!("block control flow, use BCF2ACF if not supported natively")
    }

    /// Attaches an else block to the current if block
    fn else_(&mut self) {
        todo!("block control flow, use BCF2ACF if not supported natively")
    }

    /// Ends the current block
    fn end(&mut self) {
        todo!("block control flow, use BCF2ACF if not supported natively")
    }

    /// Creates a loop
    fn loop_(&mut self) {
        todo!("block control flow, use BCF2ACF if not supported natively")
    }

    /// Break from the current loop
    fn break_(&mut self) {
        todo!("block control flow, use BCF2ACF if not supported natively")
    }

    /// Continue loop iteration
    fn continue_(&mut self) {
        todo!("block control flow, use BCF2ACF if not supported natively")
    }

    /// Conditional break from the current loop
    fn cbreak(&mut self, condition: Value) {
        self.if_(condition);
        self.break_();
        self.end();
    }

    /// Conditional continue loop iteration
    fn ccontinue(&mut self, condition: Value) {
        self.if_(condition);
        self.continue_();
        self.end();
    }
}
