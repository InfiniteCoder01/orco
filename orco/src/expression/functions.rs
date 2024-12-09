use super::*;

/// A function call. Operators should also get converted to this
pub trait FunctionCall {
    // /// Get the function we are calling
    // fn callee(&self) -> &SymbolRef<dyn symbol::Function>;
}

#[debug_display]
impl std::fmt::Display for &dyn FunctionCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write!(
        //     f,
        //     "{}()",
        //     self.callee()
        //         .object()
        //         .map_or("<???>".to_owned(), |function| function
        //             .try_read()
        //             .unwrap()
        //             .name()
        //             .into_owned())
        // )
        todo!();
    }
}
