use super::*;

/// Variable declaration expression. Creates one or more variables, each one can be initialized or not
pub trait VariableDeclaration {
    /// Get the variables declared
    fn variables(&self) -> DynIter<(Type, CowStr, Option<Expression>)>;
    /// Version of [`VariableDeclaration::variables`] that yields mutable references
    fn variables_mut(&mut self) -> DynIter<(Type, CowStr, Option<Expression<Mut>>)>;
}

#[debug_display]
impl std::fmt::Display for &dyn VariableDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "let ")?;
        for (index, (ty, name, value)) in self.variables().enumerate() {
            if index > 0 {
                write!(f, ",\n    ")?;
            }
            write!(f, "{}: {}", name, ty)?;
            if let Some(value) = value {
                write!(f, " = {}", value)?;
            }
        }
        Ok(())
    }
}
