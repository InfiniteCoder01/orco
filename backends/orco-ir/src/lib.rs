//! Intermediate representation backend for orco. Does
//! not compile to anything, just a way to store the code
//! See [Backend]
#![warn(missing_docs)]

/// Code generation and actual IR
pub mod codegen;

/// Intermediate representation for code
pub mod ir;

mod forwarding;

/// The heart storage
#[derive(Debug, Default)]
pub struct Backend {
    /// Type aliases
    pub types: scc::HashMap<orco::Symbol, orco::Type>,
    /// Function declarations
    pub functions: scc::HashMap<orco::Symbol, orco::types::FunctionSignature>,
    /// Definitions
    pub function_definitions: scc::HashMap<orco::Symbol, ir::Body>,
}

impl Backend {
    #[allow(missing_docs)]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// If `ty` is a type alias, will be replaced by what is aliased.
    /// Inner aliases (f.e. struct field types) are not replaced!
    pub fn inline_type_aliases(&self, mut ty: orco::Type) -> orco::Type {
        while let orco::Type::Symbol(name) = ty {
            ty = self
                .types
                .get_sync(&name)
                .unwrap_or_else(|| panic!("undeclared type {name}"))
                .clone()
        }
        ty
    }
}

impl orco::DeclarationBackend for Backend {
    fn function(
        &self,
        name: orco::Symbol,
        params: Vec<(Option<String>, orco::Type)>,
        return_type: Option<orco::Type>,
        attrs: orco::attrs::FunctionAttributes,
    ) {
        self.functions
            .insert_sync(
                name,
                orco::types::FunctionSignature {
                    params,
                    return_type,
                    attrs,
                },
            )
            .unwrap_or_else(|_| panic!("function {name} is already declared"))
    }

    fn type_(&self, name: orco::Symbol, ty: orco::Type) {
        self.types
            .insert_sync(name, ty)
            .unwrap_or_else(|_| panic!("type {name} is already declared"))
    }
}

impl orco::CodegenBackend for Backend {
    fn cg_function(&self, name: orco::Symbol) -> impl orco::codegen::BodyCodegen {
        codegen::Codegen::new(self, name)
    }
}

impl std::fmt::Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = Ok(());
        self.types.iter_sync(|name, ty| {
            result = writeln!(f, "type {name} = {ty};");
            result.is_ok()
        });
        result?;

        writeln!(f)?;
        self.functions.iter_sync(|name, sig| {
            let body = self.function_definitions.get_sync(name);
            result = write!(f, "{}fn {name}{sig}", sig.attrs);
            if result.is_ok() {
                match body {
                    Some(body) => result = writeln!(f, " {}\n", body.get()),
                    None => result = writeln!(f, ";"),
                }
            }
            result.is_ok()
        });
        result
    }
}
