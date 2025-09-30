use crate::Backend;

pub mod primitives;
pub mod ty;
pub use ty::Type;

#[derive(Clone, Debug)]
pub struct FunctionSignature {
    pub name: String,
    pub params: Vec<Type>,
    pub ret: Type,
}

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
        let mut sig = FunctionSignature {
            name: crate::escape(name),
            params: Vec::with_capacity(params.len()),
            ret: Type::from(return_type),
        };

        write!(decl, "{} {}(", sig.ret, sig.name).unwrap();
        for (idx, (name, ty)) in params.iter().enumerate() {
            if idx > 0 {
                decl.push_str(", ");
            }
            let ty = Type::from(ty);
            write!(decl, "{ty}",).unwrap();
            sig.params.push(ty);
            if let Some(name) = name {
                write!(decl, " {}", crate::escape(*name)).unwrap();
            }
        }
        decl.push_str(")");
        self.decls.insert(name, decl);
        self.sigs.insert(name, sig);
    }
}
