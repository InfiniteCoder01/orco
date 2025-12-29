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
            OT::Integer(size) => match size {
                orco::IntegerSize::Bits(bits) => {
                    assert!(
                        [8, 16, 32, 64].contains(&bits),
                        "invalid or unsupported integer bit width {bits}"
                    );

                    // TODO: __int128_t
                    write!(f, "int{bits}_t")
                }
                orco::IntegerSize::Size => write!(f, "ssize_t"),
            },
            OT::Unsigned(size) => match size {
                orco::IntegerSize::Bits(bits) => {
                    assert!(
                        [8, 16, 32, 64].contains(&bits),
                        "invalid or unsupported integer bit width {bits}"
                    );

                    // TODO: unsigned __int128_t
                    write!(f, "uint{bits}_t")
                }
                orco::IntegerSize::Size => write!(f, "size_t"),
            },
            OT::Float(size) => match size {
                32 => write!(f, "float"),
                64 => write!(f, "double"),
                size => {
                    // TODO: f16 and f128
                    panic!("invalid or unsupported floating point type size {size} bits")
                }
            },
            OT::Bool => write!(f, "bool"),
            OT::Symbol(sym) => write!(f, "{}", backend.escape(*sym)),

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
                write!(f, "}}")
            }
            OT::Error => write!(f, "<error-type>"),
        }?;
        if let Some(name) = name {
            write!(f, " {name}")?;
        }
        Ok(())
    }
}
