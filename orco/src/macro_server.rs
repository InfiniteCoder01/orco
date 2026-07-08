use crate::{Symbol, Type};

/// Stores macros and handles their expansion.
/// EXPERIMENTAL
#[derive(Debug, Default)]
pub struct MacroServer<'a> {
    /// A map from macro name to it's function
    pub macros: scc::HashMap<Symbol, Macro<'a>>,
    /// All `call_once` macros
    call_once: scc::HashSet<(Symbol, Vec<Type>)>,
}

impl<'a> MacroServer<'a> {
    /// Registers a macro to be invoked later. If `call_once` is set,
    /// the macro will only be invoked once (used for generics).
    pub fn macro_(&self, name: Symbol, func: impl Fn(&[Type]) + Send + Sync + 'a, call_once: bool)
    where
        Self: 'a,
    {
        self.macros
            .insert_sync(
                name,
                Macro {
                    func: Box::new(func) as _,
                    call_once,
                },
            )
            .unwrap_or_else(|(name, _)| panic!("macro {name} already exists"));
    }

    /// Invokes a macro defined by [`Self::macro_`]
    pub fn invoke_macro(&self, name: Symbol, args: &[Type]) {
        let macro_ = self
            .macros
            .get_sync(&name)
            .unwrap_or_else(|| panic!("macro {name} could not be found"));
        if macro_.call_once && self.call_once.insert_sync((name, args.to_vec())).is_err() {
            return;
        }
        (macro_.func)(args);
    }
}

/// A single macro wrapper
pub struct Macro<'a> {
    /// Function to call when macro is invoked
    pub func: Box<dyn Fn(&[Type]) + Send + Sync + 'a>,
    /// Should macro parameters be hashed? See [`crate::DeclarationBackend::macro_`]
    pub call_once: bool,
}

impl std::fmt::Debug for Macro<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Macro")
            .field("call_once", &self.call_once)
            .finish()
    }
}
