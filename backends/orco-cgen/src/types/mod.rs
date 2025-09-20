use crate::Backend;

pub mod primitives;

fn write_type(to: &mut impl std::fmt::Write, ty: &orco::Type) {
    use orco::Type;
    match ty {
        Type::Symbol(sym) => write!(to, "{}", sym).unwrap(),
        Type::Error => write!(to, "<error>").unwrap(),
    }
}

impl orco::DeclarationBackend for Backend {
    fn declare_function(
        &mut self,
        name: orco::Symbol,
        params: &[(Option<orco::Symbol>, orco::Type)],
        return_type: &orco::Type,
    ) {
        use std::fmt::Write;
        let mut decl = String::new();

        write_type(&mut decl, return_type);
        write!(decl, " {}(", crate::escape(name)).unwrap();
        let mut first = true;
        for (name, ty) in params {
            if !first {
                write!(decl, ", ").unwrap();
            } else {
                first = false;
            }
            write_type(&mut decl, ty);
            if let Some(name) = name {
                write!(decl, " {}", crate::escape(*name)).unwrap();
            }
        }
        write!(decl, ")").unwrap();
        self.decls.insert(name, decl);
    }
}

// impl Backend {
//     pub fn convert_type(&self, ty: &ob::Type) -> tm::TypeBuilder {
//         match ty {
//             ob::Type::Symbol(symbol) => {
//                 tm::Type::new(tamago::BaseType::TypeDef(symbol.as_str().to_owned()))
//             }
//         }
//     }

//     pub fn build_type(&self, ty: &ob::Type) -> tm::Type {
//         self.convert_type(ty).build()
//     }
// }
