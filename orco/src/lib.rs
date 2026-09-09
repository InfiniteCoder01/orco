#![warn(missing_docs)]
#![doc = include_str!("../../README.md")]

pub use papaya;
pub use sinter;
pub use sinter::IStr as Symbol;

/// Type enums
pub mod types;
pub use types::Type;

/// Attributes are a way to pass information about symbols to the backend
pub mod attrs;

/// Body IR
pub mod ir;
pub use ir::{Body, FmtBody};

mod transforms;

use papaya::HashMap;
use std::sync::RwLock;

/// A single compilation unit.
/// Note: Be careful with mutating the types,
/// as [`Body`] caches them.
#[derive(Debug, Default)]
pub struct Module {
    /// Type declarations (aliases).
    pub types: HashMap<Symbol, RwLock<TypeAlias>>,
    /// Function declarations.
    pub functions: HashMap<Symbol, RwLock<Function>>,
}

impl Module {
    #[allow(missing_docs)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get type alias by name.
    pub fn get_ty<'a>(
        &self,
        name: Symbol,
        guard: &'a impl papaya::Guard,
    ) -> std::sync::RwLockReadGuard<'a, TypeAlias> {
        self.types
            .get(&name, guard)
            .unwrap_or_else(|| panic!("undelcared type {name}"))
            .read()
            .unwrap()
    }

    /// Get symbol by name.
    pub fn get_symbol<'a>(
        &self,
        name: Symbol,
        guard: &'a impl papaya::Guard,
    ) -> std::sync::RwLockReadGuard<'a, Function> {
        self.functions
            .get(&name, guard)
            .unwrap_or_else(|| panic!("undelcared function {name}"))
            .read()
            .unwrap()
    }

    /// Replaces the type alias by it's value until can't anymore.
    /// Reveals the true identity of the type.
    pub fn inline_ty(&self, mut ty: Type) -> Type {
        let guard = self.types.guard();
        while let Type::Symbol(name, generics) = ty {
            ty = self.get_ty(name, &guard).instantiate(&generics);
        }

        ty
    }
}

impl std::fmt::Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (name, alias) in self.types.pin().iter() {
            let alias = alias.read().unwrap();
            writeln!(
                f,
                "type {name}{} = {};",
                types::fmt_generic_params(&alias.generics),
                &alias.type_
            )?;
        }

        writeln!(f)?;

        for (name, func) in self.functions.pin().iter() {
            let func = func.read().unwrap();
            write!(f, "{}fn {name}{}", func.attrs, func)?;
            if let Some(body) = &func.body {
                writeln!(f, " {}\n", FmtBody(self, body))?;
            } else {
                writeln!(f, ";")?;
            }
        }

        Ok(())
    }
}

/// Type declaration statement.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeAlias {
    /// Type parameters.
    pub generics: Vec<Symbol>,
    /// The type we alias.
    pub type_: Type,
}

impl TypeAlias {
    /// Instantiate the type with a set of generic parameters.
    /// Shorthand for calling [`Type::instantiate`].
    pub fn instantiate(&self, generics: &[Type]) -> Type {
        self.type_
            .copy_instantiate(&self.generics.iter().copied().zip(generics).collect())
    }
}

/// Function decl & body
#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    /// Type parameters.
    pub generics: Vec<Symbol>,

    /// Parameter types with optional names.
    pub params: Vec<(Option<String>, Type)>,
    /// Return type.
    pub return_type: Option<Type>,
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
