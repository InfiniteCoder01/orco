#![warn(missing_docs)]
#![doc = include_str!("../../README.md")]

pub use sinter;
pub use sinter::IStr as Symbol;

/// Code generation, outside of declaration
pub mod codegen;
pub use codegen::CodegenBackend;

/// Type enums
pub mod types;
pub use types::Type;

/// Attributes are a way to pass information about symbols to the backend
pub mod attrs;

/// See [`MacroServer`]
pub mod macro_server;
pub use macro_server::MacroServer;

/// Declare items before defining them.
/// Think of it as an interface to generate C headers.
pub trait DeclarationBackend: Sync {
    /// Declare a function (does not have to be defined within this linker unit).
    /// Set `return_type` to [None] if require no return value.
    fn function(
        &self,
        name: Symbol,
        params: Vec<(Option<String>, Type)>,
        return_type: Option<Type>,
        attrs: attrs::FunctionAttributes,
    );

    /// Declre a type alias, should be used to declare compound types as well
    fn type_(&self, name: Symbol, ty: Type);
}
