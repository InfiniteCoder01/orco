//! C transpilation backend for orco

use std::collections::HashMap;

pub use tamago;

use orco::backend as ob;
use tamago as tm;

pub mod codegen;
pub mod types;

pub struct Backend {
    pub function_decls: HashMap<ob::Symbol, tm::Function>,
    pub function_defs: Vec<tm::Function>,
}

impl Backend {
    pub fn new() -> Self {
        Self {
            function_decls: HashMap::new(),
            function_defs: Vec::new(),
        }
    }

    pub fn build(self) -> tm::Scope {
        fn include(header: &str) -> tm::GlobalStatement {
            tm::GlobalStatement::Include(tm::IncludeBuilder::new_system_with_str(header).build())
        }
        let mut scope = tm::Scope::new()
            .global_statement(include("stdint.h"))
            .global_statement(include("stddef.h"))
            .global_statement(include("stdbool.h"))
            .new_line()
            .build();

        for (_, decl) in self.function_decls {
            scope.global_stmts.push(tm::GlobalStatement::Function(decl));
        }

        for def in self.function_defs {
            scope.global_stmts.push(tm::GlobalStatement::Function(def));
        }

        scope
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
        .replace(['.', ':', '/', '-'], "_")
}
