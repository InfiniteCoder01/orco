use super::*;

/// Block expression, contains multiple expressions (something along { expr1; expr2; })
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    pub fn infer_types(&mut self, _target_type: &Type, type_inference: &TypeInferenceInfo) {
        for expression in &mut self.expressions {
            expression.infer_types(&Type::Unit, type_inference);
        }
    }
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{")?;
        for expression in &self.expressions {
            write!(f, "{}", indent::indent_all_by(4, format!("{expression}")))?;
            if !expression.is_block() {
                write!(f, ";")?;
            }
            writeln!(f)?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}
