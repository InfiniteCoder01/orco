/// Single instruction can be thought of a node in the AST-like IR,
/// with it's children being flat written into a list of instructions right after.
/// See [`super::Body::instructions`].
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum Instruction {
    /// Signed integer constant.
    IConst(i32, crate::types::IntegerSize),
    /// Unsigned integer constant.
    UConst(u32, crate::types::IntegerSize),
    /// Float constant.
    FConst(f32, u16),
    /// Bool constant.
    BConst(bool),

    /// Load a global value (function, variable, etc.).
    Global(super::SymbolId),
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
    /// Returns the value (if any).
    Return(bool),
    /// Intrinsic. See [`super::Intrinsic`].
    Intrinsic(super::Intrinsic),
    /// Error value.
    Error,
}

impl Instruction {
    /// Number of arguments to follow this instruction with in
    /// [`super::Body::instructions`]
    pub fn arg_count(self) -> u32 {
        match self {
            Self::IConst(..) | Self::UConst(..) | Self::FConst(..) | Self::BConst(..) => 0,

            Self::Global(..) => 0,
            Self::Var(..) => 0,
            Self::Field(..) => 1,
            Self::Assign => 2,

            Self::AcfLabel(..) => 0,
            Self::AcfJump(..) => 0,
            Self::AcfCJump(..) => 1,

            Self::Call(args) => args + 1,
            Self::Return(has_value) => has_value as _,
            Self::Intrinsic(intr) => intr.arg_count(),
            Self::Error => 0,
        }
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IConst(value, size) => write!(f, "{value}_i{size}"),
            Self::UConst(value, size) => write!(f, "{value}_u{size}"),
            Self::FConst(value, size) => write!(f, "{value}_f{size}"),
            Self::BConst(value) => write!(f, "{value}"),

            Self::Global(symbol) => write!(f, "{symbol}"),
            Self::Var(id) => write!(f, "?{id}"),
            Self::Field(idx) => write!(f, "field_{idx}"),
            Self::Assign => write!(f, "assign"),

            Self::AcfLabel(label) => write!(f, "label {label}"),
            Self::AcfJump(label) => write!(f, "jump {label}"),
            Self::AcfCJump(label) => write!(f, "cjump {label}"),

            Self::Call(_) => write!(f, "call"),
            Self::Return(..) => write!(f, "return"),
            Self::Intrinsic(intr) => intr.fmt(f),
            Self::Error => write!(f, "error"),
        }
    }
}
