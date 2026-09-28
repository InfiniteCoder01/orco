/// A thin wrapper around [`orco::Type`] for formatting it as a C type.
/// Because C loves types to influence suffixes (aka arrays and function pointers),
/// also wraps optional name (variable name, parameter name, type name in typedef)
#[allow(missing_docs)]
pub struct FmtType<'a> {
    pub ty: &'a orco::Type,
    pub constant: bool,
    pub name: Option<&'a str>,
}

/// Check if a type is unit (or unit array).
pub fn is_unit(ty: &orco::Type) -> bool {
    match ty {
        orco::Type::Unit => true,
        orco::Type::Array(ty, _) => is_unit(ty),
        _ => false,
    }
}

impl std::fmt::Display for FmtType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let FmtType { ty, constant, name } = *self;

        use orco::Type as OT;
        use orco::ty::IntegerSize as IS;

        if constant && !matches!(ty, OT::Ptr(_, _)) {
            write!(f, "const ")?;
        }

        match ty {
            OT::Unit => write!(f, "void"),
            OT::Integer(size) => match size {
                IS::Bits(bits) => {
                    assert!(
                        [8, 16, 32, 64].contains(bits),
                        "invalid or unsupported integer bit width {bits}"
                    );

                    // TODO: __int128_t
                    write!(f, "int{bits}_t")
                }
                IS::Size => write!(f, "ssize_t"),
            },
            OT::Unsigned(size) => match size {
                IS::Bits(bits) => {
                    assert!(
                        [8, 16, 32, 64].contains(bits),
                        "invalid or unsupported integer bit width {bits}"
                    );

                    // TODO: unsigned __int128_t
                    write!(f, "uint{bits}_t")
                }
                IS::Size => write!(f, "size_t"),
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
            OT::Char(false) => write!(f, "char"),
            OT::Char(true) => write!(f, "wchar_t"),
            OT::Symbol(sym, generics) => {
                assert!(
                    generics.is_empty(),
                    "generics type encountered in C backend ({ty}), did you forget to monomorphize types?",
                );
                write!(f, "{}", crate::cname(*sym))
            }

            OT::Array(ty, sz) => {
                return write!(
                    f,
                    "{}[{sz}]",
                    FmtType {
                        ty,
                        constant: false,
                        name
                    }
                );
            }
            OT::Struct { fields } if fields.iter().all(|(_, ty)| is_unit(ty)) => {
                write!(f, "struct")?;
                if let Some(name) = name {
                    write!(f, " {name}")?;
                }
                write!(f, " {{}}")
            }
            OT::Struct { fields } => {
                write!(f, "struct")?;
                if let Some(name) = name {
                    write!(f, " {name}")?;
                }
                writeln!(f, " {{")?;
                for (idx, (name, ty)) in fields.iter().enumerate() {
                    if is_unit(ty) {
                        continue;
                    }
                    writeln!(
                        f,
                        "  {};",
                        FmtType {
                            ty,
                            constant: false,
                            name: Some(
                                name.as_deref()
                                    .map_or_else(
                                        || format!("_{idx}").into(),
                                        std::borrow::Cow::Borrowed
                                    )
                                    .as_ref()
                            )
                        }
                    )?;
                }
                write!(f, "}}")
            }
            OT::Ptr(ty, pointee_mutable) => {
                return write!(
                    f,
                    "{}",
                    FmtType {
                        ty,
                        constant: !*pointee_mutable,
                        name: Some(
                            format!(
                                "*{}{}",
                                match constant {
                                    true => "const ",
                                    false => "",
                                },
                                name.unwrap_or_default()
                            )
                            .trim_end()
                        )
                    }
                );
            }
            OT::FnPtr {
                params,
                return_type,
            } => {
                return write!(
                    f,
                    "{}",
                    FmtType {
                        ty: return_type,
                        constant: false,
                        name: Some(&format!(
                            "{}({})",
                            name.unwrap_or_default(),
                            params
                                .iter()
                                .filter(|ty| is_unit(ty))
                                .map(|ty| FmtType {
                                    ty,
                                    constant: false,
                                    name
                                }
                                .to_string())
                                .collect::<Vec<_>>()
                                .join(", ")
                        )),
                    }
                );
            }
            OT::Param(name) => panic!("generic param #{name} in a C type"),
            OT::Error => write!(f, "<error-type>"),
        }?;
        if let Some(name) = name {
            write!(f, " {name}")?;
        }
        Ok(())
    }
}
