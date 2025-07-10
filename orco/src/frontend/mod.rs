use crate::backend::DeclarationBackend;

/// Single source root, can be a Rust crate or a single C file.
/// orco packages will be composed of sources of (possibly) different languages
pub trait Source {
    /// Declare all symbols from this source using a declaration backend
    fn declare<DB: DeclarationBackend>(&self, backend: &mut DB);
}
