mod primitives;

/// A thin wrapper around [`orco::Type`] for formatting it as a C type.
/// Because C loves types to influence postfixes (aka arrays and function pointers),
/// also wraps optional name (variable name, parameter name, type name in typedef)
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FmtType<'a>(pub &'a orco::Type, pub Option<&'a str>);

impl std::fmt::Display for FmtType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use orco::Type as OT;
        match self.0 {
            OT::Symbol(sym) => write!(f, "{}", crate::escape(*sym))?,
            OT::Array(ty, sz) => return write!(f, "{}[{sz}]", FmtType(ty, self.1)),
            OT::Struct(fields) => {
                writeln!(f, "struct {{")?;
                for (name, ty) in fields {
                    writeln!(f, "  {};", FmtType(ty, Some(&crate::escape(*name))))?;
                }
                write!(f, "}}")?;
            }
            OT::Error => write!(f, "<error-type>")?,
        }
        if let Some(name) = self.1 {
            write!(f, " {name}")?;
        }
        Ok(())
    }
}
