//! C transpilation backend for orco
#![warn(missing_docs)]

use std::collections::HashMap;

pub mod codegen;
pub mod declare;

#[derive(Debug, Default)]
pub struct Backend {
    pub decls: HashMap<orco::Symbol, String>,
    /// Function signatures, Used for function definitions
    pub sigs: HashMap<orco::Symbol, declare::FunctionSignature>,
    pub defs: std::sync::RwLock<Vec<String>>, // TODO: use a lock-free data struct
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
        writeln!(f).unwrap();
        for def in self.defs.read().unwrap().iter() {
            writeln!(f, "{def}\n")?;
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
