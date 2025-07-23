use crate::backend::{DeclarationBackend, DefinitionBackend};

/// Single source root, can be a Rust crate or a single C file.
/// orco packages will be composed of sources of (possibly) different languages
pub trait Source {
    /// Declare all symbols from this source using a declaration backend.
    /// Should generate enough for a C header file
    fn declare(&self, backend: &mut impl DeclarationBackend);

    /// Define all symbols from this source using a definition backend
    fn define(&self, backend: &mut impl DefinitionBackend);
}
