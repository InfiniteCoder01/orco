use super::*;

/// A single function parameter
pub trait FunctionParameter {
    /// Parameter name, optiona;
    fn name(&self) -> Option<CowStr>;
    /// Parameter type
    fn r#type(&self) -> Type;
}

#[debug_display]
impl std::fmt::Display for &dyn FunctionParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {}",
            self.name().as_ref().map_or("_", std::borrow::Cow::as_ref),
            self.r#type()
        )
    }
}

/// Function signature. Contains all the typing information about this function
pub trait FunctionSignature {
    /// Get the function parameters
    fn parameters(&self) -> DynIter<&dyn FunctionParameter>;
    /// Version of [`Function::parameters`] that yields mutable references
    fn parameters_mut(&mut self) -> DynIter<&mut dyn FunctionParameter>;
    /// Get the return type of the function
    fn return_type(&self) -> Type;
}

#[debug_display]
impl std::fmt::Display for &dyn FunctionSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}) -> {}",
            self.parameters().fold(String::new(), |acc, param| {
                if acc.is_empty() {
                    format!("{}", param)
                } else {
                    format!("{}, {}", acc, param)
                }
            }),
            self.return_type()
        )
    }
}

/// This is a function definition
pub trait Function {
    /// Returns the name of the function
    fn name(&self) -> CowStr;
    /// Get the signature of this function
    fn signature(&self) -> &dyn FunctionSignature;
    /// Version of [`Function::signature`] that returns mutable reference
    fn signature_mut(&mut self) -> &mut dyn FunctionSignature;
    /// Returns the body of the function. Value of this expression could be used as a return value
    fn body(&self) -> Expression;
    /// Version of [`Function::body`] that returns mutable reference
    fn body_mut(&mut self) -> Expression<Mut>;
}

#[debug_display]
impl std::fmt::Display for &dyn Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fn {} {} {}", self.name(), self.signature(), self.body())
    }
}
