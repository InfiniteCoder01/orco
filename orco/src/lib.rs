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

/// Some backend building blocks
pub mod impls;

/// Declare items before defining them.
/// Think of it as an interface to generate C headers.
pub trait DeclarationBackend<'a>: Sync {
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

    /// Registers a macro. `func` will now be
    /// called with macro args on [`Self::invoke_macro`].
    /// If `call_once` is set, will only be called once for a set of args,
    /// useful for generics
    /// EXPERIMENTAL
    fn macro_(
        &self,
        name: Symbol,
        func: impl Fn(&Self, &[Type]) + Send + Sync + 'a,
        call_once: bool,
    );

    /// Invokes a macro registered by [`Self::macro_`]
    /// EXPERIMENTAL
    fn invoke_macro(&self, name: Symbol, args: &[Type]);
}
