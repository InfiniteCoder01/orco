/// A thin wrapper around [`orco::Type`] for formatting it as a C type
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FmtType<'a>(pub &'a orco::Type);

impl std::fmt::Display for FmtType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use orco::Type as OT;
        match self.0 {
            OT::Symbol(sym) => write!(f, "{}", crate::escape(*sym)),
            OT::Array(ty, size) => todo!(),
            OT::Struct(fields) => todo!(),
            OT::Error => write!(f, "<error-type>"),
        }
    }
}
