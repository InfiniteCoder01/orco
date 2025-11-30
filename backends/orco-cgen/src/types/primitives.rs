use crate::Backend;

impl orco::PrimitiveTypeSource for Backend {
    fn bool(&self) -> orco::Type {
        orco::Type::Symbol("bool".into())
    }

    fn int(&self, size: u16, signed: bool) -> orco::Type {
        assert!(
            [8, 16, 32, 64].contains(&size),
            "invalid or unsupported integer bit width {size}"
        );
        // TODO: __int128_t
        if signed {
            orco::Type::Symbol(format!("int{size}_t").into())
        } else {
            orco::Type::Symbol(format!("uint{size}_t").into())
        }
    }

    fn size_type(&self, signed: bool) -> orco::Type {
        if signed {
            orco::Type::Symbol("ssize_t".into())
        } else {
            orco::Type::Symbol("size_t".into())
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
