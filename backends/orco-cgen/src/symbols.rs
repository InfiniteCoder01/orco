use crate::FmtType;

/// Main symbol enum
#[derive(Clone, Debug, PartialEq)]
pub enum SymbolKind {
    /// Function, see [FunctionSignature]
    Function(FunctionSignature),
    /// Type alias, aka typedef
    Type(orco::Type),
    /// Macro
    Generic {
        /// Generic param names
        params: Vec<String>,
        /// Wrapped symbol
        symbol: Box<SymbolKind>,
    },
}

/// Formats a symbol for display in C language
#[allow(missing_docs)]
pub struct FmtSymbol<'a> {
    pub backend: &'a crate::Backend,
    pub macro_context: bool,
    pub name: &'a str,
    pub kind: &'a SymbolKind,
}

impl std::fmt::Display for FmtSymbol<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let FmtSymbol {
            backend,
            macro_context,
            name,
            kind,
        } = *self;

        match kind {
            SymbolKind::Function(signature) => {
                write!(
                    f,
                    "{};",
                    FmtFunction {
                        macro_context,
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
                        macro_context,
                        ty,
                        name: Some(name)
                    }
                )
            }
            SymbolKind::Generic { params, symbol } => {
                write!(f, "#define {}(", name)?;
                for (idx, param) in params.iter().enumerate() {
                    if idx > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{param}")?;
                }
                write!(f, ")")?;

                let sym_name = std::iter::once(name)
                    .chain(params.iter().map(String::as_ref))
                    .collect::<Vec<_>>()
                    .join("##_##");

                let symbol = format!(
                    "{}",
                    FmtSymbol {
                        backend,
                        name: &sym_name,
                        kind: symbol,
                        macro_context: true,
                    }
                );
                for line in symbol.split('\n') {
                    writeln!(f, " \\")?;
                    write!(f, "{line}")?;
                }

                Ok(())
            }
        }
    }
}

/// Function signature without a name
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctionSignature {
    /// Parameter types with optional names
    pub params: Vec<(Option<String>, orco::Type)>,
    /// Return type
    pub return_type: orco::Type,
}

impl FunctionSignature {
    /// Get function pointer type for this function signature
    pub fn ptr_type(&self) -> orco::Type {
        orco::Type::FnPtr {
            params: self.params.iter().map(|(_, ty)| ty.clone()).collect(),
            return_type: Box::new(self.return_type.clone()),
        }
    }
}

/// Formats function signature
pub struct FmtFunction<'a> {
    pub macro_context: bool,
    pub name: &'a str,
    pub signature: &'a FunctionSignature,
    pub name_all_args: bool,
}

impl std::fmt::Display for FmtFunction<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let FmtFunction {
            macro_context,
            name,
            signature,
            name_all_args,
        } = *self;

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
                    macro_context,
                    ty,
                    name: match name {
                        Some(name) => Some(name.to_owned()),
                        None if name_all_args => Some(format!("_{idx}")),
                        None => None,
                    }
                    .as_deref()
                }
            )?;
        }
        write!(sig_noret, ")")?;

        FmtType {
            macro_context,
            ty: &signature.return_type,
            name: Some(&sig_noret),
        }
        .fmt(f)
    }
}
