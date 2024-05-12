use super::*;
use crate::ir::expression::Variable;

#[derive(Clone, Debug)]
/// Function signature (i.e. parameters and return type)
pub struct Signature {
    /// Function parameters
    pub args: Spanned<Vec<Variable>>,
    /// Function return type
    pub return_type: Spanned<Type>,
}

impl Signature {
    /// Create a new function signature
    pub fn new(args: Spanned<Vec<Variable>>, return_type: Spanned<Type>) -> Self {
        Self { args, return_type }
    }

    /// Get the type for this function signature
    /// Returns a function pointer
    pub fn get_type(&self) -> Type {
        Type::FunctionPointer(
            Spanned::new(
                self.args
                    .iter()
                    .map(|arg| {
                        Spanned::new(arg.r#type.lock().unwrap().clone(), arg.r#type.span.clone())
                    })
                    .collect(),
                self.args.span.clone(),
            ),
            Box::new(self.return_type.clone()),
        )
    }

    /// Format
    pub fn format(&self, f: &mut std::fmt::Formatter<'_>, name: Option<&str>) -> std::fmt::Result {
        write!(f, "fn ")?;
        if let Some(name) = name {
            write!(f, "{}", name)?;
        }
        write!(f, "(")?;
        for (index, arg) in self.args.iter().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}: {}", arg.name, arg.r#type.lock().unwrap())?;
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
