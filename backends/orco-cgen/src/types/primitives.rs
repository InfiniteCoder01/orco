use crate::Backend;

impl orco::PrimitiveTypeSource for Backend {
    fn bool(&self) -> orco::Type {
        orco::Type::Symbol("bool".into())
    }

    fn int(&self, size: orco::IntegerSize, signed: bool) -> orco::Type {
        match size {
            orco::IntegerSize::Bits(bits) => {
                assert!(
                    [8, 16, 32, 64].contains(&bits),
                    "invalid or unsupported integer bit width {bits}"
                );
                // TODO: __int128_t
                if signed {
                    orco::Type::Symbol(format!("int{bits}_t").into())
                } else {
                    orco::Type::Symbol(format!("uint{bits}_t").into())
                }
            }
            orco::IntegerSize::Size => {
                if signed {
                    orco::Type::Symbol("ssize_t".into())
                } else {
                    orco::Type::Symbol("size_t".into())
                }
            }
        }
    }

    fn float(&self, size: u16) -> orco::Type {
        // TODO: f16 and f128
        orco::Type::Symbol(
            match size {
                32 => "float",
                64 => "double",
                size => panic!("invalid or unsupported floating point type size {size} bits"),
            }
            .into(),
        )
    }
}
