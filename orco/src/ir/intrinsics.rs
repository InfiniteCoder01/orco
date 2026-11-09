/// Intrinsics are operations built into the compier.
#[repr(align(1))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Intrinsic {
    /// Adds two ints/floats.
    Add,
    /// Subtracts two ints/floats.
    Sub,
    /// Multiplies two ints/floats.
    Mul,
    /// Divides two ints/floats.
    Div,
    /// Division remainder of two ints/floats.
    Rem,
    /// Negates an int/float.
    Neg,

    /// And of two ints (bitwise) or bools (logical).
    And,
    /// Or of two ints (bitwise) or bools (logical).
    Or,
    /// Xor of two ints (bitwise) or bools (logical).
    Xor,
    /// Not of an int (bitwise) or bool (logical).
    Not,

    /// Shift an integer left by N bits.
    Shl,
    /// Shift an integer right by N bits.
    Shr,

    /// Checks if two values are equal (any values, pointers compared by address).
    Eq,
    /// Checks if two values are not equal (any values, pointers compared by address).
    Ne,
    /// Compares ints/floats (less than).
    Lt,
    /// Compares ints/floats (less than or equal).
    Le,
    /// Compares ints/floats (greater than).
    Gt,
    /// Compares ints/floats (greater than or equal).
    Ge,
}

impl Intrinsic {
    /// Returns the number of arguments this intrinsic requires.
    pub fn arg_count(self) -> u32 {
        match self {
            Self::Add => 2,
            Self::Sub => 2,
            Self::Mul => 2,
            Self::Div => 2,
            Self::Rem => 2,
            Self::Neg => 1,

            Self::And => 2,
            Self::Or => 2,
            Self::Xor => 2,
            Self::Not => 1,

            Self::Shl => 2,
            Self::Shr => 2,

            Self::Eq => 2,
            Self::Ne => 2,
            Self::Lt => 2,
            Self::Le => 2,
            Self::Gt => 2,
            Self::Ge => 2,
        }
    }

    /// Weather debug display should use infix notation for this intrinsic.
    pub fn infix(self) -> bool {
        self.arg_count() >= 2
    }

    /// For some intrinsics yields their return type,
    /// for others type must be derived from the arguments.
    pub fn type_override(self) -> Option<crate::Type> {
        use crate::Type;
        Some(match self {
            Self::Eq => Type::Bool,
            Self::Ne => Type::Bool,
            Self::Lt => Type::Bool,
            Self::Le => Type::Bool,
            Self::Gt => Type::Bool,
            Self::Ge => Type::Bool,
            _ => return None,
        })
    }
}

impl std::fmt::Display for Intrinsic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Add => write!(f, "+"),
            Self::Sub => write!(f, "-"),
            Self::Mul => write!(f, "*"),
            Self::Div => write!(f, "/"),
            Self::Rem => write!(f, "%"),
            Self::Neg => write!(f, "-"),

            Self::And => write!(f, "&"),
            Self::Or => write!(f, "|"),
            Self::Xor => write!(f, "^"),
            Self::Not => write!(f, "!"),

            Self::Shl => write!(f, "<<"),
            Self::Shr => write!(f, ">>"),

            Self::Eq => write!(f, "=="),
            Self::Ne => write!(f, "!="),
            Self::Lt => write!(f, "<"),
            Self::Le => write!(f, "<="),
            Self::Gt => write!(f, ">"),
            Self::Ge => write!(f, ">="),
        }
    }
}

impl From<Intrinsic> for super::Instr {
    fn from(value: Intrinsic) -> Self {
        Self::Intrinsic(value)
    }
}
