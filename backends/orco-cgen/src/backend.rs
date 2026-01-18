use super::*;

/// Root backend struct
#[derive(Debug, Default)]
pub struct Backend {
    /// A map from symbol to it's declaration
    pub symbols: scc::HashMap<orco::Symbol, SymbolKind>,
    /// Interned types
    interned: scc::HashSet<orco::Symbol>,
    /// Definitions
    definitions: scc::Stack<String>,
}

impl Backend {
    #[allow(missing_docs)]
    pub fn new() -> Backend {
        Self::default()
    }
}

impl BackendContext for Backend {
    fn backend(&self) -> &Backend {
        self
    }

    fn macro_context(&self) -> bool {
        false
    }

    fn symbol(&self, name: orco::Symbol, kind: SymbolKind) {
        self.symbols
            .entry_sync(name)
            .and_modify(|_| panic!("symbol {name:?} is already declared"))
            .or_insert(kind);
    }

    fn define(&self, code: String) {
        self.definitions.push(code);
    }

    fn intern_type(&self, ty: &mut orco::Type, named: bool, replace_unit: bool) {
        match ty {
            orco::Type::Array(ty, _) => {
                self.intern_type(ty.as_mut(), false, false) // TODO: More work on arrays
            }
            orco::Type::Struct(fields) if named => {
                for (_, ty) in fields {
                    self.intern_type(ty, false, false);
                }
            }
            orco::Type::Struct(fields) if !named => {
                if fields.is_empty() {
                    if replace_unit {
                        *ty = orco::Type::Symbol("void".into());
                    }
                    return;
                }

                let sym = orco::Symbol::new(&format!("s_{}", ty.hashable_name()));
                let ty = std::mem::replace(ty, orco::Type::Symbol(sym));
                if self.interned.insert_sync(sym).is_ok() {
                    use orco::DeclarationBackend as _;
                    self.type_(sym, ty);
                }
            }
            _ => (),
        }
    }
}

impl orco::DeclarationBackend for Backend {
    fn function(
        &self,
        name: orco::Symbol,
        mut params: Vec<(Option<String>, orco::Type)>,
        mut return_type: orco::Type,
    ) {
        for (_, ty) in &mut params {
            self.intern_type(ty, false, false);
        }
        self.intern_type(&mut return_type, false, true);
        self.symbol(
            name,
            SymbolKind::Function(symbols::FunctionSignature {
                params,
                return_type,
            }),
        );
    }

    fn type_(&self, name: orco::Symbol, mut ty: orco::Type) {
        self.intern_type(&mut ty, true, false);
        self.symbol(name, SymbolKind::Type(ty));
    }

    fn generic(&self, params: Vec<String>) -> impl orco::DeclarationBackend {
        generics::Wrapper {
            backend: self,
            params,
        }
    }
}

impl orco::CodegenBackend for crate::Backend {
    fn function(&self, name: orco::Symbol) -> impl orco::codegen::BodyCodegen {
        codegen::Codegen::new(self, name)
    }
}

impl std::fmt::Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "#include <stdint.h>")?;
        writeln!(f, "#include <stddef.h>")?;
        writeln!(f, "#include <stdbool.h>")?;
        writeln!(f)?;

        let mut result = Ok(());
        self.symbols.iter_sync(|name, sym| {
            let sym = format!(
                "{}",
                symbols::FmtSymbol {
                    backend: self,
                    name: &symname(*name, false),
                    kind: sym,
                    macro_context: false,
                }
            );
            result = writeln!(
                f,
                "{}{}",
                sym,
                if sym.lines().count() > 1 { "\n" } else { "" }
            );
            result.is_ok()
        });
        result?;

        for def in self.definitions.iter(&scc::Guard::new()) {
            writeln!(f, "{def}")?;
        }

        Ok(())
    }
}
