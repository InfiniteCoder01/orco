#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

pub use sinter;
pub use sinter::IStr as Symbol;

/// Code generation, outside of declaration
pub mod codegen;
pub use codegen::CodegenBackend;

/// Type of a variable, constant, part of a function signature, etc.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
    /// Signed integer
    Integer(IntegerSize),
    /// Unsigned integer
    Unsigned(IntegerSize),
    /// IEEE (or not) floating point number with set number of bits
    Float(u16),
    /// A boolean. Should be 1 byte I guess...
    Bool,
    /// Just a symbol
    Symbol(Symbol),

    /// An array type (`Type[size]`)
    Array(Box<Type>, usize),
    /// A struct, aka a collection of field-type pairs.
    Struct(Vec<(Option<String>, Type)>),
    /// Pointer (with mutability)
    Ptr(Box<Type>, bool),
    /// Function pointer
    FnPtr {
        /// Types of parameters
        params: Vec<Type>,
        /// Return type
        return_type: Box<Type>,
    },
    /// An error type
    Error,
}

impl Type {
    /// Returns a type name that could be used for hashing, mangling
    /// and human-facing names
    pub fn hashable_name(&self) -> String {
        let fmt_size = |size| match size {
            IntegerSize::Bits(bits) => bits.to_string(),
            IntegerSize::Size => "size".to_owned(),
        };

        match self {
            Type::Integer(size) => format!("i{}", fmt_size(*size)),
            Type::Unsigned(size) => format!("u{}", fmt_size(*size)),
            Type::Float(size) => format!("f{size}"),
            Type::Bool => "bool".to_owned(),

            Type::Symbol(sym) => sym.to_string(),
            Type::Array(ty, len) => format!("{}[{len}]", ty.hashable_name()),
            Type::Struct(fields) => fields
                .iter()
                .map(|(_, ty)| ty.hashable_name())
                .collect::<Vec<_>>()
                .join(" "),
            Type::Ptr(ty, mutable) => {
                format!(
                    "*{} {}",
                    match mutable {
                        true => "mut",
                        false => "const",
                    },
                    ty.hashable_name()
                )
            }
            Type::FnPtr {
                params,
                return_type,
            } => format!(
                "({}) -> {}",
                params
                    .iter()
                    .map(Type::hashable_name)
                    .collect::<Vec<_>>()
                    .join(", "),
                return_type.hashable_name()
            ),
            Type::Error => "<error>".to_owned(),
        }
    }
}

/// Integer size
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IntegerSize {
    /// Number of bits. Not sure if non-powers-of-two
    /// should be supported. Maybe even non-whole bytes (ex. u6 for 6 bit unsigned integer)
    Bits(u16),
    /// Kinda like `usize`/`isize` in rust or `size_t`/`ssize_t` in C
    Size,
}

/// Declare items before defining them.
/// Think of it as an interface to generate C headers.
pub trait DeclarationBackend: Sync {
    /// Declare a function (does not have to be defined within this linker unit)
    fn function(&self, name: Symbol, params: Vec<(Option<String>, Type)>, return_type: Type);

    /// Declre a type alias, should be used to declare compound types as well
    fn type_(&self, name: Symbol, ty: Type);

    /// Returns a backend that wraps every symbol in a macro with generic params.
    /// Nestable. Appends #param to all symbol names.
    /// e.g. if you provide `param1, param2` as params and then declare symbol `sym`,
    /// the backend is going to generate a symbol `sym#param1#param2`.
    /// Same syntax is used if you want to use generic params. So in `another#param1`, `param1`
    /// is going to be substituted for the parameter value during instantiation
    fn generic(&self, params: Vec<String>) -> impl DeclarationBackend;
}
