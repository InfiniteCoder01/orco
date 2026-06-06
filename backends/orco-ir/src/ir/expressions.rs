use orco::Type;
use orco::codegen as oc;

/// Alternate version of [`oc::Place`] that uses
/// [Expression] instead of [`oc::Value`].
/// See also [`crate::codegen::Codegen::cvt_place`]
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Place {
    /// Just variable access
    Variable(oc::Variable),
    /// Global symbol access
    Global(orco::Symbol),
    /// Pointer dereference
    Deref(Box<Expression>),
    /// Field access, using 0-based field index
    Field(Box<Place>, usize),
}

impl Place {
    /// Returns type and mutability
    pub fn get_type(&self, backend: &crate::Backend, body: &super::Body) -> (Type, bool) {
        match self {
            Self::Variable(variable) => {
                let variable = body.get_variable(*variable);
                (variable.ty.clone(), true)
            }
            Self::Global(name) => (
                backend
                    .functions
                    .get_sync(name)
                    .unwrap_or_else(|| panic!("undeclared symbol {name}"))
                    .ptr_type(),
                false,
            ),
            Self::Deref(expr) => match backend.inline_type_aliases(expr.get_type(backend, body)) {
                Type::Ptr(ty, mutable) => (*ty, mutable),
                ty => panic!("trying to dereference non-pointer type {ty}"),
            },
            Self::Field(place, idx) => {
                let (ty, mutable) = place.get_type(backend, body);
                (
                    match backend.inline_type_aliases(ty) {
                        Type::Struct { mut fields } => fields.swap_remove(*idx).1,
                        ty => panic!("trying to access field _{idx} on non-struct type {ty}"),
                    },
                    mutable,
                )
            }
        }
    }
}

impl std::fmt::Display for Place {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Place::Variable(var) => write!(f, "_{}", var.0),
            Place::Global(name) => write!(f, "{name}"),
            Place::Deref(expr) => write!(f, "*{expr}"),
            Place::Field(place, idx) => write!(f, "{place}._{idx}"),
        }
    }
}

/// Basic expressions
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Expression {
    /// See [`oc::BodyCodegen::iconst`]
    IConst(i128, orco::types::IntegerSize),
    /// See [`oc::BodyCodegen::uconst`]
    UConst(u128, orco::types::IntegerSize),
    /// See [`oc::BodyCodegen::fconst`]
    FConst(f64, u16),
    /// See [`oc::BodyCodegen::fconst`]
    BConst(bool),
    /// See [`oc::BodyCodegen::read`]
    Read(Place),
    /// See [`oc::BodyCodegen::reference`]
    Reference(Place, bool),
    /// See [`oc::BodyCodegen::call`].
    Call(Box<Expression>, Vec<Expression>),

    /// See [`oc::BodyCodegen::intrinsics`]
    Intrinsic(super::Intrinsic),
}

impl Expression {
    /// Get type of the value this statement produces
    pub fn get_type(&self, backend: &crate::Backend, body: &super::Body) -> Type {
        match self {
            Self::IConst(_, size) => Type::Integer(*size),
            Self::UConst(_, size) => Type::Unsigned(*size),
            Self::FConst(_, size) => Type::Float(*size),
            Self::BConst(_) => Type::Bool,
            Self::Read(place) => place.get_type(backend, body).0,
            Self::Reference(place, mutable) => {
                Type::Ptr(Box::new(place.get_type(backend, body).0), *mutable)
            }
            Self::Call(func, ..) => match func.get_type(backend, body) {
                Type::FnPtr { return_type, .. } => {
                    return_type.map_or(Type::Error, |ty| *ty.clone())
                }
                _ => Type::Error,
            },

            Self::Intrinsic(intrinsic) => intrinsic.get_type(backend, body),
        }
    }
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IConst(value, size) => write!(f, "{value} as i{size}")?,
            Self::UConst(value, size) => write!(f, "{value} as u{size}")?,
            Self::FConst(value, size) => write!(f, "{value} as f{size}")?,
            Self::BConst(value) => write!(f, "{value}")?,
            Self::Read(place) => write!(f, "{place}")?,
            Self::Reference(place, mutable) => {
                write!(f, "&{} {place}", if *mutable { "mut" } else { "const" })?
            }
            Self::Call(func, args) => {
                write!(f, "{func}(")?;
                for (idx, arg) in args.iter().enumerate() {
                    if idx > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{arg}")?;
                }
                write!(f, ")")?;
            }

            Self::Intrinsic(intrinsic) => write!(f, "{intrinsic}")?,
        }
        Ok(())
    }
}
