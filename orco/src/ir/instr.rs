/// Single instruction can be thought of a node in the AST-like IR,
/// with it's children being flat written into a list of instructions right after.
/// See [`super::Body::instructions`].
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Instruction {
    /// Unit constant. See [`crate::Type::Unit`]
    Unit,
    /// Signed integer constant.
    IConst(i64, crate::ty::IntegerSize),
    /// Unsigned integer constant.
    UConst(u64, crate::ty::IntegerSize),
    /// Float constant.
    FConst(f64, u8),
    /// Bool constant.
    BConst(bool),
    /// Character constant.
    CConst(u32, bool),

    /// Load a global value (function, variable, etc.).
    Global(crate::SymbolId, thin_vec::ThinVec<crate::Type>),
    /// Load the variable.
    Var(super::VariableId),
    /// Access a field at index.
    Field(u32),
    /// Assign the value to a place last expression references.
    Assign,

    /// Just places a label here, allowing jump to this point.
    AcfLabel(super::LabelId),
    /// Unconditionally jump to a label.
    AcfJump(super::LabelId),
    /// Conditionally jump to a label.
    AcfCJump(super::LabelId),

    /// Call a function with a specified number of arguments.
    Call(u32),
    /// Returns the value.
    Return,
    /// Intrinsic. See [`super::Intrinsic`].
    Intrinsic(super::Intrinsic),

    /// Cast ints/ptrs/floats/chars/bools.
    Cast(Box<crate::Type>),
    /// Transmute any types (aka reinterpret/bit cast).
    Transmute(Box<crate::Type>),

    /// Error value.
    Error,
}

impl Instruction {
    /// Number of arguments to follow this instruction with in
    /// [`super::Body::instructions`]
    pub fn arg_count(&self) -> u32 {
        match self {
            Self::Unit
            | Self::IConst(..)
            | Self::UConst(..)
            | Self::FConst(..)
            | Self::BConst(..)
            | Self::CConst(..) => 0,

            Self::Global(..) => 0,
            Self::Var(..) => 0,
            Self::Field(..) => 1,
            Self::Assign => 2,

            Self::AcfLabel(..) => 0,
            Self::AcfJump(..) => 0,
            Self::AcfCJump(..) => 1,

            Self::Call(args) => args + 1,
            Self::Return => 1,
            Self::Intrinsic(intr) => intr.arg_count(),

            Self::Cast(..) => 1,
            Self::Transmute(..) => 1,

            Self::Error => 0,
        }
    }
}
