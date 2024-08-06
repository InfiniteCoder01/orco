use super::*;
use std::pin::Pin;

#[derive(Clone, Debug)]
/// Function signature (i.e. parameters and return type)
pub struct Signature {
    /// Function parameters
    pub args: Spanned<Vec<Pin<Box<VariableDeclaration>>>>,
    /// Function return type
    pub return_type: Spanned<Type>,
}

impl Signature {
    /// Create a new function signature
    pub fn new(
        args: Spanned<Vec<Pin<Box<VariableDeclaration>>>>,
        return_type: Spanned<Type>,
    ) -> Self {
        Self { args, return_type }
    }

    /// Get the type for this function signature
    /// Returns a function pointer type
    pub fn get_type(&self) -> Type {
        Type::FunctionPointer(
            Spanned::new(
                self.args
                    .iter()
                    .map(|arg| {
                        Spanned::new(
                            arg.r#type.try_lock().unwrap().clone(),
                            arg.r#type.span.clone(),
                        )
                    })
                    .collect(),
                self.args.span.clone(),
            ),
            Box::new(self.return_type.clone()),
        )
    }
}

impl std::fmt::Display for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        for (index, arg) in self.args.iter().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}: {}", arg.name, arg.r#type.try_lock().unwrap())?;
        }
        write!(f, ")")?;
        write!(f, " -> {}", *self.return_type)?;
        Ok(())
    }
}
