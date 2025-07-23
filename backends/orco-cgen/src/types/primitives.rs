use crate::{Backend, ob};

impl ob::PrimitiveTypeSource for Backend {
    fn unit(&self) -> ob::Type {
        ob::Type::Symbol(ob::Symbol::new_static(&"void"))
    }

    fn bool(&self) -> ob::Type {
        ob::Type::Symbol(ob::Symbol::new_static(&"bool"))
    }

    fn int(&self, size: u16, signed: bool) -> ob::Type {
        if signed {
            ob::Type::Symbol(ob::Symbol::new(format!("int{size}_t")))
        } else {
            ob::Type::Symbol(ob::Symbol::new(format!("uint{size}_t")))
        }
    }

    fn size_type(&self, signed: bool) -> ob::Type {
        if signed {
            ob::Type::Symbol(ob::Symbol::new_static(&"ssize_t"))
        } else {
            ob::Type::Symbol(ob::Symbol::new_static(&"size_t"))
        }
    }

    fn float(&self, size: u16) -> ob::Type {
        ob::Type::Symbol(ob::Symbol::new_static(match size {
            32 => &"float",
            64 => &"double",
            _ => panic!("invalid or unsupported floating point type size {size} bits"),
        }))
    }
}
