//! C transpilation backend for orco

pub use tamago;

use orco::backend as ob;
use tamago as tm;

pub mod types;

pub struct Backend(tm::Scope);
impl ob::DeclarationBackend for Backend {
    fn function(
        &mut self,
        name: ob::Symbol,
        params: &[(Option<ob::Symbol>, ob::Type)],
        return_type: &ob::Type,
    ) {
        let mut function =
            tm::Function::new(name.to_string(), self.build_type(return_type)).build();
        for (name, ty) in params {
            function.params.push(
                tm::Parameter::new(
                    name.map_or("", |name| name.as_str()).to_string(),
                    self.build_type(ty),
                )
                .build(),
            );
        }
        self.0
            .global_stmts
            .push(tm::GlobalStatement::Function(function));
    }
}

impl Backend {
    pub fn new() -> Self {
        Self(tm::Scope::new().build())
    }

    pub fn build(self) -> tm::Scope {
        self.0
    }
}

impl Default for Backend {
    fn default() -> Self {
        Self::new()
    }
}

pub fn escape(symbol: ob::Symbol) -> String {
    symbol
        .as_str()
        .replace("::", "_")
        .replace(&['.', ':', '/', '-'], "_")
}
