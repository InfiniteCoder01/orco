mod primitives;

/// A thin wrapper around [`orco::Type`] for formatting it as a C type.
/// Because C loves types to influence postfixes (aka arrays and function pointers),
/// also wraps optional name (variable name, parameter name, type name in typedef)
pub struct FmtType<'a, B: crate::BackendContext>(
    pub &'a B,
    pub &'a orco::Type,
    pub Option<&'a str>,
);

impl<B: crate::BackendContext> std::fmt::Display for FmtType<'_, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let FmtType(backend, ty, name) = *self;

        use orco::Type as OT;
        match ty {
            OT::Symbol(sym) => write!(f, "{}", backend.escape(*sym))?,
            OT::Array(ty, sz) => return write!(f, "{}[{sz}]", FmtType(backend, ty, name)),
            OT::Struct(fields) => {
                writeln!(f, "struct {{")?;
                for (name, ty) in fields {
                    writeln!(
                        f,
                        "  {};",
                        FmtType(backend, ty, Some(&backend.escape(*name)))
                    )?;
                }
                write!(f, "}}")?;
            }
            OT::Error => write!(f, "<error-type>")?,
        }
        if let Some(name) = name {
            write!(f, " {name}")?;
        }
        Ok(())
    }
}
