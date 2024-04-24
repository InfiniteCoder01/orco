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

    /// Get the type this block evaluates to
    pub fn get_type(&self, root: &crate::ir::Module) -> Type {
        for expression in &self.expressions {
            if expression.get_type(root) == Type::Never {
                return Type::Never;
            }
        }
        Type::Unit
    }

    /// Infer types
    pub fn infer_and_check_types(&mut self, _target_type: &Type, type_info: &TypeInferenceInfo) {
        for expression in &mut self.expressions {
            expression.infer_and_check_types(&Type::Unit, type_info);
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
