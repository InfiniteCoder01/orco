use crate::FmtType;
use orco::Symbol;

/// Main symbol enum
#[derive(Clone, Debug, PartialEq)]
pub enum SymbolKind {
    /// Function, see [FunctionSignature]
    Function {
        signature: FunctionSignature,
        body: Option<String>,
    },
    /// Type alias, aka typedef
    Type(orco::Type),
    /// Macro
    Generic {
        params: Vec<Symbol>,
        symbol: Box<SymbolKind>,
    },
}

/// Formats a symbol for display in C language
pub struct FmtSymbol<'a, B: crate::BackendContext>(pub &'a B, pub &'a str, pub &'a SymbolKind);
impl<B: crate::BackendContext> std::fmt::Display for FmtSymbol<'_, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let FmtSymbol(backend, name, kind) = *self;

        match kind {
            SymbolKind::Function { signature, body } => {
                write!(
                    f,
                    "{}",
                    FmtFunction {
                        backend,
                        name,
                        signature,
                        name_all_args: true,
                    }
                )?;
                if let Some(body) = body {
                    write!(f, " {body}")
                } else {
                    write!(f, ";")
                }
            }
            SymbolKind::Type(ty) => {
                write!(f, "typedef {};", FmtType(backend, ty, Some(name)))
            }
            SymbolKind::Generic { params, symbol } => {
                write!(f, "#define {}(", name)?;
                for (idx, param) in params.iter().enumerate() {
                    if idx > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", backend.escape(*param))?;
                }
                write!(f, ")")?;

                let sym_name = std::iter::once(name.to_owned())
                    .chain(params.iter().map(|param| backend.escape(*param)))
                    .collect::<Vec<_>>()
                    .join("##_##");

                let symbol = format!("{}", FmtSymbol(backend, &sym_name, symbol));
                for line in symbol.split('\n') {
                    writeln!(f, " \\")?;
                    write!(f, "{line}")?;
                }

                Ok(())
            }
        }
    }
}

/// Function signature using C [Type]s without a name
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctionSignature {
    /// Parameter types with optional names
    pub params: Vec<(Option<Symbol>, orco::Type)>,
    /// Return type
    pub return_type: orco::Type,
}

/// Formats function signature
pub struct FmtFunction<'a, B: crate::BackendContext> {
    backend: &'a B,
    name: &'a str,
    signature: &'a FunctionSignature,
    name_all_args: bool,
}

impl<'a, B: crate::BackendContext> std::fmt::Display for FmtFunction<'a, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut sig_noret = self.name.to_owned();

        use std::fmt::Write as _;
        write!(&mut sig_noret, "(")?;
        for (idx, (name, ty)) in self.signature.params.iter().enumerate() {
            if idx > 0 {
                write!(sig_noret, ", ")?;
            }
            write!(
                sig_noret,
                "{}",
                FmtType(
                    self.backend,
                    ty,
                    match name {
                        Some(name) => Some(self.backend.escape(*name)),
                        None if self.name_all_args => Some(format!("_{idx}")),
                        None => None,
                    }
                    .as_ref()
                    .map(String::as_str)
                )
            )?;
        }
        write!(sig_noret, ")")?;

        FmtType(self.backend, &self.signature.return_type, Some(&sig_noret)).fmt(f)
    }
}
