use super::*;

/// This is a function definition
pub trait Function {
    /// Returns the name of the function
    fn name(&self) -> std::borrow::Cow<str>;
    /// Returns the body of the function. Value of this expression could be used as a return value
    fn body(&self) -> Expression;
    /// Version of [`Function::body`] that returns mutable reference
    fn body_mut(&mut self) -> Expression<Mut>;
    /// Get the return type of the function
    fn return_type(&self) -> Type;
}

#[debug_display]
impl std::fmt::Display for &dyn Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "fn {} () -> {} {}",
            self.name(),
            self.return_type(),
            self.body()
        )
    }
}
