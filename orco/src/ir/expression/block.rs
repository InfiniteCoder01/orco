use super::*;

/// Block expression, contains multiple expressions (something along { expr1; expr2; })
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Block {
    /// Block content
    pub expressions: Vec<Expression>,
}

impl Block {
    /// Create a new block
    pub fn new(expressions: Vec<Expression>) -> Self {
        Self { expressions }
    }

    /// Infer types
    pub fn infer_types(&mut self, _target_type: &Type, return_type: &Type) {
        for expression in &mut self.expressions {
            expression.infer_types(&Type::Unit, return_type);
        }
    }
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{")?;
        for expression in &self.expressions {
            writeln!(f, "{};", indent::indent_all_by(4, format!("{expression}")))?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}
