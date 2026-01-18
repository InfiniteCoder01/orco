//! C transpilation backend for orco.
//! Also used to generate C headers and is generally the reference for other backends
//! See [Backend]
// TODO: ABI
// TODO: Extra type interning
#![warn(missing_docs)]

/// Type formatting & other things
pub mod types;
use types::FmtType;

/// Symbol container types
pub mod symbols;
pub use symbols::SymbolKind;

/// Code generation, used to generate function bodies.
pub mod codegen;
pub use codegen::Codegen;

/// Backend struct
mod backend;
pub use backend::Backend;
/// Generics wrapper
mod generics;

/// Backend context. Either [Backend] or [`generics::Wrapper`]
pub trait BackendContext {
    /// Get the original [Backend]
    fn backend(&self) -> &Backend;

    /// Returns true if generated code will be in a macro (aka token pasting is allowed)
    fn macro_context(&self) -> bool;

    /// Get symbol name within this context. See [symname]
    fn symname(&self, symbol: orco::Symbol) -> String {
        symname(symbol, self.macro_context())
    }

    /// Declares a symbol
    fn symbol(&self, name: orco::Symbol, kind: SymbolKind);

    /// Defines a symbol
    fn define(&self, code: String);

    /// Intern the following type and it's insides.
    fn intern_type(&self, ty: &mut orco::Type, named: bool, replace_unit: bool);
}

/// Get the name of the symbol used in generated code
fn symname(symbol: orco::Symbol, macro_context: bool) -> String {
    // TODO: Needs work

    // Take only the method name, not the path
    // FIXME: conflicts...
    let symbol = &symbol[symbol.rfind([':', '.']).map_or(0, |i| i + 1)..];

    let mut symbol = symbol.replace(|c: char| !c.is_ascii_alphanumeric() && c != '#', "_");
    if symbol.chars().next().is_none_or(|c| c.is_ascii_digit()) {
        symbol.insert(0, '_');
    }

    symbol.replace('#', if macro_context { "##_##" } else { "_" })
}
