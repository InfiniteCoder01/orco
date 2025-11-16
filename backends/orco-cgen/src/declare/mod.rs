use crate::Backend;

mod primitives;
mod ty;
pub use ty::Type;

/// Function signature using C [Type]s without a name
/// (see [`super::Declaration`] for name and generics).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctionSignature {
    /// Parameter types with optional names
    pub params: Vec<(Type, Option<String>)>,
    /// Return type
    pub ret: Type,
}

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
                    write!(f, "{ty}")?;
                    if let Some(name) = name {
                        write!(f, " {name}")?;
                    }
                }
                write!(f, ");")
            }
            DeclarationKind::Type(ty) => {
                write!(f, "typedef {ty} {name};", name = self.name)
            }
        }
    }
}

/// All kinds of declarations
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DeclarationKind {
    /// Function decl, see [FunctionSignature]
    Function(FunctionSignature),
    /// Typedef
    Type(Type),
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
            .and_modify(|_| panic!("symbol {name:?} is already declared"))
            .or_insert(Declaration {
                name: crate::escape(name),
                kind: DeclarationKind::Function(sig),
            });
    }

    fn declare_type(&self, name: orco::Symbol, ty: orco::Type) {
        self.decls
            .entry_sync(name)
            .and_modify(|_| panic!("symbol {name:?} is already declared"))
            .or_insert(Declaration {
                name: crate::escape(name),
                kind: DeclarationKind::Type(self.convert_type(&ty)),
            });
    }
}
