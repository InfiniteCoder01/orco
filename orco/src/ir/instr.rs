/// Single instruction can be thought of a node in the AST-like IR,
/// with it's children being flat written into a list of instructions right after.
/// See [`super::Body::instructions`]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum Instruction {
    /// Signed integer constant
    IConst(i128, crate::types::IntegerSize),
    /// Unsigned integer constant
    UConst(u128, crate::types::IntegerSize),
    /// Float constant
    FConst(f64, u16),
    /// Bool constant
    BConst(bool),

    /// Load the variable
    Var(super::VariableId),
    /// Access a field at index
    Field(u32),
    /// Assign the value to a place last expression references
    Assign,

    /// See [`AcfInstruction`]
    Acf(AcfInstruction),

    /// Returns the value (if any)
    Return(bool),
    /// Error value
    Error,
}

impl Instruction {
    pub fn arg_count(self) -> u32 {
        match self {
            Self::IConst(..) | Self::UConst(..) | Self::FConst(..) | Self::BConst(..) => 0,

            Self::Var(..) => 0,
            Self::Field(..) => 1,
            Self::Assign => 2,

            Self::Acf(AcfInstruction::Label(..)) => 0,
            Self::Acf(AcfInstruction::Jump(..)) => 0,
            Self::Acf(AcfInstruction::CJump(..)) => 1,

            Self::Return(has_value) => has_value as _,
            Self::Error => 0,
        }
    }

    pub fn has_value(self) -> bool {
        match self {
            Self::IConst(..) | Self::UConst(..) | Self::FConst(..) | Self::BConst(..) => true,

            Self::Var(..) => true,
            Self::Field(..) => true,
            Self::Assign => false,

            Self::Acf(..) => false,

            Self::Return(..) => false,
            Self::Error => true,
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

            Self::Var(id) => write!(f, "?{id}"),
            Self::Field(idx) => write!(f, "field_{idx}"),
            Self::Assign => write!(f, "assign"),

            Self::Acf(instr) => instr.fmt(f),

            Self::Return(..) => write!(f, "return"),
            Self::Error => write!(f, "error"),
        }
    }
}

/// Arbitrary control flow instructions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AcfInstruction {
    /// Just places a label here, allowing jump to this point.
    Label(super::LabelId),
    /// Unconditionally jump to a label.
    Jump(super::LabelId),
    /// Conditionally jump to a label.
    CJump(super::LabelId),
}

impl std::fmt::Display for AcfInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Label(label) => write!(f, "label {label}"),
            Self::Jump(label) => write!(f, "jump {label}"),
            Self::CJump(label) => write!(f, "cjump {label}"),
        }
    }
}

impl From<AcfInstruction> for Instruction {
    fn from(instr: AcfInstruction) -> Self {
        Self::Acf(instr)
    }
}
