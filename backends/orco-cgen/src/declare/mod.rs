use crate::Backend;

mod primitives;
mod ty;
pub use ty::FmtType;

/// Function signature using C [Type]s without a name
/// (see [`super::Declaration`] for name and generics).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctionSignature {
    /// Parameter types with optional names
    pub params: Vec<(Option<orco::Symbol>, orco::Type)>,
    /// Return type
    pub return_type: orco::Type,
}

/// A single item declaration
#[derive(Clone, Debug, PartialEq)]
pub struct Declaration {
    /// Name for the symbol, escaped using [`crate::escape`]
    pub name: orco::Symbol,
    /// See [DeclarationKind]
    pub kind: DeclarationKind,
}

/// All kinds of declarations
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DeclarationKind {
    /// Function decl, see [FunctionSignature]
    Function(FunctionSignature),
    /// Type alias, aka typedef
    Type(orco::Type),
}

impl std::fmt::Display for Declaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            DeclarationKind::Function(sig) => {
                write!(
                    f,
                    "{ret} {name}(",
                    ret = FmtType(&sig.return_type),
                    name = crate::escape(self.name),
                )?;
                for (idx, (name, ty)) in sig.params.iter().enumerate() {
                    if idx > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", FmtType(ty))?;
                    if let Some(name) = name {
                        write!(f, " {}", crate::escape(*name))?;
                    }
                }
                write!(f, ");")
            }
            DeclarationKind::Type(ty) => {
                write!(
                    f,
                    "typedef {ty} {name};",
                    ty = FmtType(ty),
                    name = crate::escape(self.name),
                )
            }
        }
    }
}

impl orco::DeclarationBackend for Backend {
    fn declare_function(
        &self,
        name: orco::Symbol,
        params: Vec<(Option<orco::Symbol>, orco::Type)>,
        return_type: orco::Type,
    ) {
        self.decls
            .entry_sync(name)
            .and_modify(|_| panic!("symbol {name:?} is already declared"))
            .or_insert(Declaration {
                name,
                kind: DeclarationKind::Function(FunctionSignature {
                    params,
                    return_type,
                }),
            });
    }

    fn declare_type(&self, name: orco::Symbol, ty: orco::Type) {
        self.decls
            .entry_sync(name)
            .and_modify(|_| panic!("symbol {name:?} is already declared"))
            .or_insert(Declaration {
                name,
                kind: DeclarationKind::Type(ty),
            });
    }
}
