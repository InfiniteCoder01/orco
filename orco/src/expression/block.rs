use super::*;

/// Block is a collection of expressions, some may call it a "scope"
pub trait Block {
    /// Get the expressions
    fn expressions(&self) -> DynIter<Expression>;
    /// Version of [`Block::expressions`] that yields mutable references
    fn expressions_mut(&mut self) -> DynIter<Expression<Mut>>;
}

impl std::fmt::Display for &dyn Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{")?;
        for expression in self.expressions() {
            writeln!(f, "{}", indent::indent_all_by(4, format!("{expression};")))?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}
