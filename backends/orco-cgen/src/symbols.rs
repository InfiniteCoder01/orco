use crate::FmtType;

/// Function signature using C [Type]s without a name
/// (see [`super::Declaration`] for name and generics).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctionSignature {
    /// Parameter types with optional names
    pub params: Vec<(Option<orco::Symbol>, orco::Type)>,
    /// Return type
    pub return_type: orco::Type,
}

/// Main symbol enum
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SymbolKind {
    /// Function, see [FunctionSignature]
    Function(FunctionSignature),
    /// Type alias, aka typedef
    Type(orco::Type),
}

/// Formats a symbol for display in C language
pub struct FmtSymbol<'a>(pub orco::Symbol, pub &'a SymbolKind);
impl std::fmt::Display for FmtSymbol<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let FmtSymbol(name, kind) = self;
        let name = *name;

        match kind {
            SymbolKind::Function(sig) => {
                write!(
                    f,
                    "{ret} {name}(",
                    ret = FmtType(&sig.return_type),
                    name = crate::escape(name),
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
            SymbolKind::Type(ty) => {
                write!(
                    f,
                    "typedef {ty} {name};",
                    ty = FmtType(ty),
                    name = crate::escape(name),
                )
            }
        }
    }
}
