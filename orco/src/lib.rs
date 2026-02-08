#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

pub use sinter;
pub use sinter::IStr as Symbol;

/// Code generation, outside of declaration
pub mod codegen;
pub use codegen::CodegenBackend;

/// Type enums
pub mod types;
pub use types::Type;

/// Attributes are a way to pass information about a function to the backend
pub mod attrs;

/// Declare items before defining them.
/// Think of it as an interface to generate C headers.
pub trait DeclarationBackend: Sync {
    /// Declare a function (does not have to be defined within this linker unit)
    fn function(
        &self,
        name: Symbol,
        params: Vec<(Option<String>, Type)>,
        return_type: Type,
        attrs: attrs::FunctionAttributes,
    );

    /// Declre a type alias, should be used to declare compound types as well
    fn type_(&self, name: Symbol, ty: Type);

    /// Returns a backend that wraps every symbol in a macro with generic params.
    /// Nestable. Appends #param to all symbol names.
    /// e.g. if you provide `param1, param2` as params and then declare symbol `sym`,
    /// the backend is going to generate a symbol `sym#param1#param2`.
    /// Same syntax is used if you want to use generic params. So in `another#param1`, `param1`
    /// is going to be substituted for the parameter value during instantiation
    fn generic(&self, params: Vec<String>) -> impl DeclarationBackend;
}
