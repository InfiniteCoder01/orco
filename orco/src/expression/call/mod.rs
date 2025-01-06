/// Calls are everything that happens.
/// `break`, `return`, adding two numbers
/// together are all function calls
pub struct Call {
    /// Function to call
    pub function: crate::ArcLock<crate::expression::Function>,
    /// Args for the function
    pub args: Vec<crate::Expression>,
}

impl std::fmt::Display for Call {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}(",
            self.function
                .read()
                .unwrap()
                .name()
                .unwrap_or("<unnamed function>")
        )?;
        for (index, arg) in self.args.iter().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }
            arg.fmt(f)?;
        }
        write!(f, ")")?;
        Ok(())
    }
}
