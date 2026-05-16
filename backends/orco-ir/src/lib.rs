//! Intermediate representation backend for orco. Does
//! not compile to anything, just a way to store the code
//! See [Backend]
#![warn(missing_docs)]

/// Code generation and actual IR
pub mod codegen;

/// Intermediate representation for code
pub mod ir;

/// The heart storage
#[derive(Debug, Default)]
pub struct Backend<'a> {
    /// Type aliases
    pub types: scc::HashMap<orco::Symbol, orco::Type>,
    /// Function declarations
    pub functions: scc::HashMap<orco::Symbol, orco::types::FunctionSignature>,
    /// Definitions
    pub function_definitions: scc::HashMap<orco::Symbol, ir::Body>,
    /// Macro server, default impl
    pub macros: orco::impls::MacroServer<'a>,
}

impl Backend<'_> {
    #[allow(missing_docs)]
    pub fn new() -> Self {
        Self::default()
    }
}

impl<'a> orco::DeclarationBackend<'a> for Backend<'a> {
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

    fn macro_(
        &self,
        name: orco::Symbol,
        func: impl Fn(&[orco::Type]) + Send + Sync + 'a,
        call_once: bool,
    ) {
        self.macros.macro_(name, func, call_once)
    }

    fn invoke_macro(&self, name: orco::Symbol, args: &[orco::Type]) {
        self.macros.invoke_macro(name, args)
    }
}

impl orco::CodegenBackend for Backend<'_> {
    fn function(&self, name: orco::Symbol) -> impl orco::codegen::BodyCodegen {
        codegen::Codegen::new(self, name)
    }
}

impl std::fmt::Display for Backend<'_> {
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
