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
                let mut sig_noret = crate::escape(name);
                use std::fmt::Write as _;
                write!(&mut sig_noret, "(")?;
                for (idx, (name, ty)) in sig.params.iter().enumerate() {
                    if idx > 0 {
                        write!(sig_noret, ", ")?;
                    }
                    write!(
                        sig_noret,
                        "{}",
                        FmtType(ty, name.map(crate::escape).as_ref().map(String::as_str))
                    )?;
                }
                write!(sig_noret, ")")?;

                write!(f, "{};", FmtType(&sig.return_type, Some(&sig_noret)))
            }
            SymbolKind::Type(ty) => {
                write!(f, "typedef {};", FmtType(ty, Some(&crate::escape(name))),)
            }
        }
    }
}
