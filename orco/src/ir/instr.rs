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
    /// Assign the value to a place last expression references
    Assign,

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
            Self::Assign => 2,

            Self::Return(has_value) => has_value as _,
            Self::Error => 0,
        }
    }

    pub fn has_value(self) -> bool {
        match self {
            Self::IConst(..) | Self::UConst(..) | Self::FConst(..) | Self::BConst(..) => true,

            Self::Var(..) => true,
            Self::Assign => false,

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
            Self::Assign => write!(f, "assign"),

            Self::Return(..) => write!(f, "return"),
            Self::Error => write!(f, "error"),
        }
    }
}
