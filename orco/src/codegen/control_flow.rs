use super::Value;

/// A label ID. See [`AcfCodegen::label`]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Label(pub usize);

/// Arbitrary control flow instructions, such as jumps.
/// Warning: Not all codegens implement arbitrary control flow
pub trait AcfCodegen {
    /// Allocates a label to be placed later
    fn alloc_label(&mut self) -> Label;

    /// Places a label in the current position.
    fn label(&mut self, label: Label);

    /// Jump to a label.
    /// See [`AcfCodegen::label`]
    fn jump(&mut self, label: Label);

    /// Jumps if condition is true.
    /// See [`AcfCodegen::label`]
    fn cjump(&mut self, condition: Value, label: Label);
}

/// Block control flow (somewhat traditional/wasm style).
pub trait BcfCodegen {
    /// Starts a block that will only be executed if the condition is met
    fn if_(&mut self, condition: Value);
    /// Attaches an else block to the current if block
    fn else_(&mut self);
    /// Ends the current block
    fn end(&mut self);

    /// Creates a loop
    fn loop_(&mut self);
    /// Break from the current loop
    fn break_(&mut self);
    /// Continue loop iteration
    fn continue_(&mut self);

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
