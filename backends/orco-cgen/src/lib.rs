//! C transpilation backend for orco

use std::collections::HashMap;

// pub mod codegen;
pub mod types;

#[derive(Clone, Debug, Default)]
pub struct Backend {
    pub decls: HashMap<orco::Symbol, String>,
}

impl Backend {
    pub fn new() -> Self {
        Self::default()
    }
}

impl std::fmt::Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "#include <stdint.h>")?;
        writeln!(f, "#include <stddef.h>")?;
        writeln!(f, "#include <stdbool.h>")?;
        writeln!(f)?;

        for (_, decl) in &self.decls {
            writeln!(f, "{decl};")?;
        }
        Ok(())
    }
}

pub fn escape(symbol: orco::Symbol) -> String {
    symbol
        .as_str()
        .replace("::", "_")
        .replace(['.', ':', '/', '-'], "_")
}
