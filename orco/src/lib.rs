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

/// Declare items before defining them.
/// Think of it as an interface to generate C headers (uh oh generics...).
/// For adding generic params, see [`Type::Param`]
pub trait DeclarationBackend {
    /// Declare a function (does not have to be defined within this linker unit).
    /// Set `return_type` to [None] if require no return value.
    /// Specializations declared during codegen
    fn function(
        &self,
        name: Symbol,
        generic_params: Vec<Type>,
        params: Vec<(Option<String>, Type)>,
        return_type: Option<Type>,
        attrs: attrs::FunctionAttributes,
    );

    /// Declre a type alias, can be used to declare compound types as well.
    /// Specializations declared using this function as well
    fn type_(&self, name: Symbol, generic_params: Vec<Type>, ty: Type);
}
