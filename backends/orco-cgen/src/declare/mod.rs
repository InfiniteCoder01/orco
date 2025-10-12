use crate::Backend;

mod primitives;
mod ty;
pub use ty::Type;

mod function;
pub use function::FunctionSignature;

/// A single item declaration
#[derive(Clone, Debug, PartialEq)]
pub struct Declaration {
    /// Name for the symbol, escaped using [`crate::escape`]
    pub name: String,
    /// See [DeclarationKind]
    pub kind: DeclarationKind,
}

impl std::fmt::Display for Declaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            DeclarationKind::Function(sig) => {
                write!(f, "{ret} {name}(", ret = sig.ret, name = self.name)?;
                for (idx, (ty, name)) in sig.params.iter().enumerate() {
                    if idx > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{ty}",)?;
                    if let Some(name) = name {
                        write!(f, " {name}")?;
                    }
                }
                write!(f, ");")
            }
        }
    }
}

/// All kinds of declarations
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DeclarationKind {
    /// Function decl, see [FunctionSignature]
    Function(FunctionSignature),
}

impl orco::DeclarationBackend for Backend {
    fn declare_function(
        &self,
        name: orco::Symbol,
        params: &[(Option<orco::Symbol>, orco::Type)],
        return_type: &orco::Type,
    ) {
        let sig = FunctionSignature {
            params: params
                .into_iter()
                .map(|(name, ty)| (self.convert_type(ty), (*name).map(crate::escape)))
                .collect(),
            ret: self.convert_type(&return_type),
        };

        self.decls
            .entry_sync(name)
            .and_modify(|_| panic!("function {name:?} is already declared!"))
            .or_insert(Declaration {
                name: crate::escape(name),
                kind: DeclarationKind::Function(sig),
            });
    }
}
