use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// Function signature (i.e. parameters and return type)
pub struct Signature {
    /// Function parameters
    pub args: Vec<(String, Spanned<Type>)>,
    /// Function return type
    pub return_type: Spanned<Type>,
}

impl Signature {
    /// Create a new function signature
    pub fn new(arguments: Vec<(String, Spanned<Type>)>, return_type: Spanned<Type>) -> Self {
        Self {
            args: arguments,
            return_type,
        }
    }

    /// Format
    pub fn format(&self, f: &mut std::fmt::Formatter<'_>, name: Option<&str>) -> std::fmt::Result {
        write!(f, "fn ")?;
        if let Some(name) = name {
            write!(f, "{}", name)?;
        }
        write!(f, "(")?;
        for (index, (name, r#type)) in self.args.iter().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}: {}", name, **r#type)?;
        }
        write!(f, ")")?;
        write!(f, " -> {}", *self.return_type)?;
        Ok(())
    }
}

impl std::fmt::Display for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format(f, None)
    }
}
