use orco::Type;
use orco::codegen as oc;

/// Basic instructions
#[derive(Debug, PartialEq, PartialOrd)]
pub enum Statement {
    /// See [`oc::BodyCodegen::comment`]
    Comment(String),
    /// See [`oc::BodyCodegen::assign`]
    Assign(oc::Place, oc::Value),
    /// See [`oc::BodyCodegen::iconst`]
    IConst(i128, orco::types::IntegerSize),
    /// See [`oc::BodyCodegen::uconst`]
    UConst(u128, orco::types::IntegerSize),
    /// See [`oc::BodyCodegen::fconst`]
    FConst(f64, u16),
    /// See [`oc::BodyCodegen::fconst`]
    BConst(bool),
    /// See [`oc::BodyCodegen::read`]
    Read(oc::Place),
    /// See [`oc::BodyCodegen::reference`]
    Reference(oc::Place, bool),
    /// See [`oc::BodyCodegen::call`].
    /// Additionally stores wether there is a return value
    Call(oc::Value, Vec<oc::Value>, bool),
    /// See [`oc::BodyCodegen::return`]
    Return(Option<oc::Value>),
    /// See [`oc::BodyCodegen::intrinsics`]
    Intrinsic(super::Intrinsic),
}

impl Statement {
    /// Weather this statement is an expression (it yields a value)
    pub fn is_expression(&self) -> bool {
        match self {
            Self::Comment(..) => false,
            Self::Assign(..) => false,
            Self::IConst(..) => true,
            Self::UConst(..) => true,
            Self::FConst(..) => true,
            Self::BConst(..) => true,
            Self::Read(..) => true,
            Self::Reference(..) => true,
            Self::Call(_, _, has_retval) => *has_retval,
            Self::Return(..) => false,
            Self::Intrinsic(..) => true,
        }
    }

    /// Get type of the value this statement produces
    pub fn get_type(&self, backend: &crate::Backend, body: &super::Body) -> Type {
        match self {
            Self::Comment(_) => Type::Error,
            Self::Assign(..) => Type::Error,
            Self::IConst(_, size) => Type::Integer(*size),
            Self::UConst(_, size) => Type::Unsigned(*size),
            Self::FConst(_, size) => Type::Float(*size),
            Self::BConst(_) => Type::Bool,
            Self::Read(place) => place_ty(place, backend, body).0,
            Self::Reference(place, mutable) => {
                Type::Ptr(Box::new(place_ty(place, backend, body).0), *mutable)
            }
            Self::Call(func, ..) => match body.type_of(func.0, backend) {
                Type::FnPtr { return_type, .. } => {
                    return_type.map_or(Type::Error, |ty| *ty.clone())
                }
                _ => Type::Error,
            },
            Self::Return(_) => Type::Error,
            Self::Intrinsic(intrinsic) => intrinsic.get_type(backend, body),
        }
    }
}

impl std::fmt::Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Comment(comment) => {
                for (idx, line) in comment.split('\n').enumerate() {
                    if idx > 0 {
                        writeln!(f)?;
                    }
                    write!(f, "// {line}")?;
                }
            }
            Self::Assign(target, value) => write!(f, "{target} = <{}>;", value.0)?,
            Self::IConst(value, size) => write!(f, "{value} as i{size}")?,
            Self::UConst(value, size) => write!(f, "{value} as u{size}")?,
            Self::FConst(value, size) => write!(f, "{value} as f{size}")?,
            Self::BConst(value) => write!(f, "{value}")?,
            Self::Read(place) => write!(f, "{place}")?,
            Self::Reference(place, mutable) => {
                write!(f, "&{} {place}", if *mutable { "mut" } else { "const" })?
            }
            Self::Call(func, args, _) => {
                write!(f, "{func}(")?;
                for (idx, arg) in args.iter().enumerate() {
                    if idx > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{arg}")?;
                }
                write!(f, ")")?;
            }
            Self::Return(value) => {
                write!(f, "return")?;
                if let Some(value) = value {
                    write!(f, " {value}")?;
                }
                write!(f, ";")?;
            }
            Self::Intrinsic(intrinsic) => write!(f, "{intrinsic}")?,
        }
        Ok(())
    }
}

/// Get type and mutability of [`oc::Place`]
pub fn place_ty(place: &oc::Place, backend: &crate::Backend, body: &super::Body) -> (Type, bool) {
    match place {
        oc::Place::Variable(variable) => {
            let variable = body.get_variable(*variable);
            (variable.ty.clone(), true)
        }
        oc::Place::Global(name) => (
            backend
                .functions
                .get_sync(name)
                .unwrap_or_else(|| panic!("undeclared symbol {name}"))
                .ptr_type(),
            false,
        ),
        oc::Place::Deref(value) => {
            match backend.inline_type_aliases(body.type_of(value.0, backend)) {
                Type::Ptr(ty, mutable) => (*ty, mutable),
                ty => panic!("trying to dereference non-pointer type {ty}"),
            }
        }
        oc::Place::Field(place, idx) => {
            let (ty, mutable) = place_ty(place, backend, body);
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
