use orco::Type;
use orco::codegen as oc;

/// Basic instructions
#[derive(Debug, PartialEq, PartialOrd)]
pub enum Statement {
    /// See [`orco::codegen::BodyCodegen::comment`]
    Comment(String),
    /// See [`orco::codegen::BodyCodegen::assign`]
    Assign(oc::Place, oc::Value),
    /// See [`orco::codegen::BodyCodegen::iconst`]
    IConst(i128, orco::types::IntegerSize),
    /// See [`orco::codegen::BodyCodegen::uconst`]
    UConst(u128, orco::types::IntegerSize),
    /// See [`orco::codegen::BodyCodegen::fconst`]
    FConst(f64, u16),
    /// See [`orco::codegen::BodyCodegen::fconst`]
    BConst(bool),
    /// See [`orco::codegen::BodyCodegen::read`]
    Read(oc::Place),
    /// See [`orco::codegen::BodyCodegen::reference`]
    Reference(oc::Place, bool),
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
