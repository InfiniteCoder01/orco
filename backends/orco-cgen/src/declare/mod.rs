use crate::Backend;

pub mod primitives;
pub mod ty;
pub use ty::Type;

impl orco::DeclarationBackend for Backend {
    fn declare_function(
        &mut self,
        name: orco::Symbol,
        params: &[(Option<orco::Symbol>, orco::Type)],
        return_type: &orco::Type,
    ) {
        assert!(
            !self.decls.contains_key(&name),
            "function {name:?} is already declared!"
        );

        use std::fmt::Write as _;
        let mut decl = String::new();

        write!(decl, "{} {}(", Type::from(return_type), crate::escape(name)).unwrap();
        let mut first = true;
        for (name, ty) in params {
            if !first {
                write!(decl, ", ").unwrap();
            } else {
                first = false;
            }
            write!(decl, "{}", Type::from(ty)).unwrap();
            if let Some(name) = name {
                write!(decl, " {}", crate::escape(*name)).unwrap();
            }
        }
        write!(decl, ")").unwrap();
        self.decls.insert(name, decl);
    }
}
