#![warn(missing_docs)]
#![doc = include_str!("../../README.md")]

pub use sinter;
pub use sinter::IStr as Symbol;

/// Type enums
pub mod types;
pub use types::Type;

/// Attributes are a way to pass information about symbols to the backend
pub mod attrs;

use papaya::HashMap;
/// A single compilation unit
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Module {
    /// Type declarations (aliases)
    pub types: HashMap<Symbol, TypeAlias>,
    /// Function declarations
    pub functions: HashMap<Symbol, Function>,
}

impl Module {
    #[allow(missing_docs)]
    pub fn new() -> Self {
        Self::default()
    }
}

impl std::fmt::Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (name, alias) in self.types.pin().iter() {
            writeln!(
                f,
                "type {name}{} = {};",
                types::fmt_generic_params(&alias.generics),
                &alias.type_
            )?;
        }

        writeln!(f)?;

        for (name, func) in self.functions.pin().iter() {
            writeln!(f, "{}fn {name}{};", func.attrs, func,)?;
        }

        Ok(())
    }
}

/// Type declaration statement
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeAlias {
    /// Type parameters
    pub generics: Vec<Symbol>,
    /// The type we alias
    pub type_: Type,
}

/// Function decl & body
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Function {
    /// Type parameters
    pub generics: Vec<Symbol>,
    /// Parameter types with optional names
    pub params: Vec<(Option<String>, Type)>,
    /// Return type
    pub return_type: Option<Type>,
    /// Function attributes
    pub attrs: crate::attrs::FunctionAttributes,
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(", types::fmt_generic_params(&self.generics))?;

        for (idx, (name, ty)) in self.params.iter().enumerate() {
            if idx > 0 {
                write!(f, ", ")?;
            }

            match name {
                Some(name) => write!(f, "{name:}: ")?,
                None => write!(f, "_{idx}: ")?,
            }

            ty.fmt(f)?;
        }

        match &self.return_type {
            Some(ty) => {
                write!(f, ") -> ")?;
                ty.fmt(f)
            }
            None => write!(f, ") -> void"),
        }
    }
}
