use crate::FmtType;
use orco::types::FunctionSignature;

/// Reference to a declaration
#[derive(Debug)]
pub enum SymbolKind {
    /// Function, see [FunctionSignature]
    Function(FunctionSignature),
    /// Type alias, aka typedef
    Type(orco::Type),
}

/// Formats a symbol for display in C language
#[allow(missing_docs)]
pub struct FmtSymbol<'a> {
    pub name: &'a str,
    pub kind: &'a SymbolKind,
}

impl std::fmt::Display for FmtSymbol<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let FmtSymbol { name, kind } = *self;

        match kind {
            SymbolKind::Function(signature) => {
                write!(
                    f,
                    "{};",
                    FmtFunction {
                        name,
                        signature,
                        name_all_args: true,
                    }
                )
            }
            SymbolKind::Type(ty) => {
                write!(
                    f,
                    "typedef {};",
                    FmtType {
                        ty,
                        constant: false,
                        name: Some(name)
                    }
                )
            }
        }
    }
}

/// Formats function signature
pub struct FmtFunction<'a> {
    /// Function name
    pub name: &'a str,
    /// Function signature
    pub signature: &'a FunctionSignature,
    /// Wether to name all args (assign placeholder names)?
    pub name_all_args: bool,
}

impl std::fmt::Display for FmtFunction<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let FmtFunction {
            name,
            signature,
            name_all_args,
        } = *self;

        use orco::attrs as oa;
        match signature.attrs.inlining {
            oa::Inlining::Never => write!(f, "__attribute__ ((noinline)) ")?,
            oa::Inlining::Auto => (),
            oa::Inlining::Hint => write!(f, "inline ")?,
            oa::Inlining::Always => write!(f, "__attribute__ ((always_inline)) ")?,
        }

        let mut sig_noret = name.to_owned();

        use std::fmt::Write as _;
        write!(&mut sig_noret, "(")?;
        for (idx, (name, ty)) in signature.params.iter().enumerate() {
            if idx > 0 {
                write!(sig_noret, ", ")?;
            }
            write!(
                sig_noret,
                "{}",
                FmtType {
                    ty,
                    constant: false,
                    name: match name {
                        Some(name) => Some(name.to_owned()),
                        None if name_all_args => Some(format!("arg{idx}")),
                        None => None,
                    }
                    .as_deref()
                }
            )?;
        }
        write!(sig_noret, ")")?;

        FmtType {
            ty: signature
                .return_type
                .as_ref()
                .unwrap_or(&orco::Type::Symbol("void".into())),
            constant: false,
            name: Some(&sig_noret),
        }
        .fmt(f)
    }
}
