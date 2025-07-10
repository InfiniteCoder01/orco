use crate::{Backend, ob, tm};

impl ob::TypeBackend for Backend {
    fn unit(&self) -> ob::Symbol {
        ob::Symbol::new_static(&"void")
    }

    fn int(&self, size: u16, signed: bool) -> ob::Symbol {
        if signed {
            ob::Symbol::new(format!("int{size}_t"))
        } else {
            ob::Symbol::new(format!("uint{size}_t"))
        }
    }

    fn size_type(&self, signed: bool) -> ob::Symbol {
        if signed {
            ob::Symbol::new_static(&"ssize_t")
        } else {
            ob::Symbol::new_static(&"size_t")
        }
    }
}

impl Backend {
    pub fn convert_type(&self, ty: &ob::Type) -> tm::TypeBuilder {
        match ty {
            ob::Type::Symbol(symbol) => {
                tm::Type::new(tamago::BaseType::TypeDef(symbol.as_str().to_owned()))
            }
        }
    }

    pub fn build_type(&self, ty: &ob::Type) -> tm::Type {
        self.convert_type(ty).build()
    }
}
