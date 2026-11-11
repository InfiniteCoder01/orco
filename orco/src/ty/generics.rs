use super::Type;

/// Type parameter.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TyParam {
    /// Optional name.
    pub name: Option<String>,
}

impl Type {
    /// Replace all instances of [`Type::Param`] with types from `generics`.
    pub fn instantiate(&mut self, generics: &[Type]) {
        match self {
            Self::Integer(..)
            | Self::Unsigned(..)
            | Self::Float(..)
            | Self::Bool
            | Self::Char(..)
            | Self::Unit => (),
            Self::Alias(_, tys) => {
                for ty in tys {
                    ty.instantiate(generics);
                }
            }

            Self::Array(ty, _) => ty.instantiate(generics),
            Self::Struct { fields } => {
                for (_, ty) in fields {
                    ty.instantiate(generics);
                }
            }
            Self::Ptr(ty, _) => ty.instantiate(generics),
            Self::FnPtr { types } => {
                for ty in types {
                    ty.instantiate(generics);
                }
            }

            Self::Param(idx) => generics[*idx as usize].clone_into(self),
            Self::Error => (),
        }
    }

    /// Same as [`Self::instantiate`], but clones the type in the process.
    pub fn copy_instantiate(&self, map: &[Type]) -> Self {
        let mut instance = self.clone();
        instance.instantiate(map);
        instance
    }

    /// Check if this type contains type params.
    pub fn has_params(&self) -> bool {
        match self {
            Self::Unit => false,
            Self::Integer(..) => false,
            Self::Unsigned(..) => false,
            Self::Float(..) => false,
            Self::Bool => false,
            Self::Char(..) => false,
            Self::Alias(_, generics) => {
                for generic in generics {
                    if generic.has_params() {
                        return true;
                    }
                }
                false
            }

            Self::Array(ty, _) => ty.has_params(),
            Self::Struct { fields } => {
                for (_, field) in fields {
                    if field.has_params() {
                        return true;
                    }
                }
                false
            }
            Self::Ptr(ty, _) => ty.has_params(),
            Self::FnPtr { types } => {
                for ty in types {
                    if ty.has_params() {
                        return true;
                    }
                }
                false
            }

            Self::Param(..) => true,
            Self::Error => false,
        }
    }
}

impl crate::Function {
    /// Instantiate this function with a set of generic args.
    /// See also [`Type::instantiate`].
    pub fn instantiate(&mut self, generics: &[Type]) {
        self.generics.clear(); // Clear the generic list as this function is no longer generic.

        for (_, ty) in &mut self.params {
            ty.instantiate(generics);
        }

        self.return_type.instantiate(generics);

        let Some(body) = &mut self.body else {
            return;
        };

        for var in &mut body.variables {
            var.ty.instantiate(generics);
        }

        for instr in &mut body.instructions {
            use crate::ir::Instr;
            match instr {
                Instr::Global(_, args) => {
                    for ty in args {
                        ty.instantiate(generics);
                    }
                }
                Instr::Cast(ty) | Instr::Transmute(ty) => ty.instantiate(generics),
                _ => (),
            }
        }
    }
}

impl std::fmt::Display for crate::Display<'_, [TyParam]> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            return Ok(());
        }

        write!(f, "<")?;
        for (idx, param) in self.iter().enumerate() {
            if idx > 0 {
                write!(f, ", ")?;
            }

            if let Some(name) = &param.name {
                name.fmt(f)?;
            }

            write!(f, "#{idx}")?;
        }
        write!(f, ">")
    }
}

impl std::fmt::Display for crate::Display<'_, [Type]> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            return Ok(());
        }

        write!(f, "<")?;
        for (idx, ty) in self.iter().enumerate() {
            if idx > 0 {
                write!(f, ", ")?;
            }
            self.project(ty).fmt(f)?;
        }
        write!(f, ">")
    }
}
