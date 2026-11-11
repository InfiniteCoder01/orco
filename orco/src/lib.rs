#![warn(missing_docs)]
#![doc = include_str!("../../README.md")]
pub use thin_vec::ThinVec;

/// See [`Module`].
pub mod module;
pub use module::{Module, Symbol, SymbolId};

/// Display wrapper struct.
mod display;
pub use display::Display;

/// Type enums.
pub mod ty;
pub use ty::{TyParam, Type};

/// Attributes are a way to pass information about symbols to the backend.
pub mod attrs;

/// Body IR.
pub mod ir;
pub use ir::Body;

mod transformations;

/// Function decl & body.
#[derive(Clone, PartialEq)]
pub struct Function {
    /// Optional name.
    pub name: Option<String>,
    /// Type parameters.
    pub generics: Vec<TyParam>,

    /// Parameter types with optional names.
    pub params: Vec<(Option<String>, Type)>,
    /// Return type.
    pub return_type: Type,
    /// Function attributes.
    pub attrs: crate::attrs::FunctionAttributes,
    /// Function body.
    pub body: Option<Body>,
}

impl Function {
    /// Generate a function body with all argument variables pre-added.
    pub fn create_def(&self) -> Body {
        let mut body = Body::new();
        for (name, ty) in self.params.iter().cloned() {
            body.variables.push(ir::VariableInfo {
                ty,
                arg: true,
                name,
            });
        }
        body
    }
}

impl std::fmt::Display for Display<'_, Function> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}fn {}{}(",
            self.attrs,
            self.name.as_deref().unwrap_or("(unnamed)"),
            self.project(&self.generics[..])
        )?;

        for (idx, (name, ty)) in self.params.iter().enumerate() {
            if idx > 0 {
                write!(f, ", ")?;
            }

            match name {
                Some(name) => write!(f, "{name:}: ")?,
                None => write!(f, "_{idx}: ")?,
            }

            self.project(ty).fmt(f)?;
        }

        write!(f, ") -> ")?;
        self.project(&self.return_type).fmt(f)?;

        if let Some(body) = &self.body {
            write!(f, " ")?;
            self.project(body).fmt(f)?;
        }

        Ok(())
    }
}
