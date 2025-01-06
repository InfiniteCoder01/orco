use super::*;

/// C function declaration and definition
pub mod function;
pub use function::FunctionDefinition;

/// C symbols
#[derive(Parse, ToTokens)]
pub enum Symbol {
    /// Function definition
    FunctionDefinition(FunctionDefinition),
}

impl Symbol {
    pub fn build(&self, ctx: &mut orco::TypeInferenceContext) -> (String, orco::Expression) {
        match self {
            Self::FunctionDefinition(function) => (
                function.name.to_string(),
                orco::Expression::Function(function.build(ctx)),
            ),
        }
    }
}
