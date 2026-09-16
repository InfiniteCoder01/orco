use crate::types::IntegerSize;

/// Intrinsics are operations built into the compier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Intrinsic {
    /// Adds two numbers. Ints and floats supported.
    Add,
    /// Subtracts two numbers. Ints and floats supported.
    Sub,
    /// Multiplies two numbers. Ints and floats supported.
    Mul,
    /// Divides two numbers. Ints and floats supported.
    Div,
    /// Modulo of the two two numbers. Ints and floats supported.
    Rem,
    /// Negate a number. Ints and floats are supported.
    Neg,

    /// And of two values. Bitwise for integers, logical for booleans.
    And,
    /// Or of two values. Bitwise for integers, logical for booleans.
    Or,
    /// Xor of two values. Bitwise for integers, logical for booleans.
    Xor,
    /// Not. Bitwise for integers, logical for booleans.
    Not,

    /// Shift a number left by N bits. Integers only.
    Shl,
    /// Shift a number right by N bits. Integers only.
    Shr,

    /// Compares two arbitrary values. Any type supported,
    /// pointers will be compared by address.
    Eq,
    /// Compares two numbers (less than). Ints and floats supported.
    Lt,
    /// Compares two numbers (less than or equal). Ints and floats supported.
    Le,
    /// Compares two numbers (greater than). Ints and floats supported.
    Gt,
    /// Compares two numbers (greater than or equal). Ints and floats supported.
    Ge,

    /// Constructs a bigger value from smaller literals (bitwise concat),
    /// useful for large constants. Type is inherited from the literals.
    /// Only argument is the number of literals to bitwise concatenate.
    AggregateLiteral(u8),
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
            Self::Lt => 2,
            Self::Le => 2,
            Self::Gt => 2,
            Self::Ge => 2,

            Self::AggregateLiteral(count) => count as _,
        }
    }

    /// Weather debug display should use infix notation for this intrinsic.
    pub fn infix(self) -> bool {
        match self {
            Self::AggregateLiteral(..) => false,
            _ if self.arg_count() == 1 => false,
            _ => true,
        }
    }

    /// For some intrinsics yields their return type,
    /// for others type must be derived from the arguments.
    pub fn type_override(self) -> Option<crate::Type> {
        use crate::Type;
        Some(match self {
            Intrinsic::Eq => Type::Bool,
            Intrinsic::Lt => Type::Bool,
            Intrinsic::Gt => Type::Bool,
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
            Self::Lt => write!(f, "<"),
            Self::Le => write!(f, "<="),
            Self::Gt => write!(f, ">"),
            Self::Ge => write!(f, ">="),

            Self::AggregateLiteral(..) => {
                write!(f, "aggregate")
            }
        }
    }
}

impl From<Intrinsic> for super::Instr {
    fn from(value: Intrinsic) -> Self {
        Self::Intrinsic(value)
    }
}

impl super::Body {
    /// Pushes an arbitrarily-sized int literal,
    /// possibly making use of [`Intrinsic::AggregateInt`].
    /// See also [`uint_literal`].
    pub fn int_literal(&mut self, mut value: i128, size: IntegerSize) {
        let mut segments = Vec::with_capacity(4);
        loop {
            segments.push((value & 0xffffffff) as i32);
            value >>= 32;
            if value == -1 || value == 0 {
                break;
            }
        }

        if segments.len() != 1 {
            self.instructions
                .push(Intrinsic::AggregateLiteral(segments.len() as _).into());
        }

        for segment in segments.into_iter().rev() {
            self.instructions.push(super::Instr::IConst(segment, size));
        }
    }

    /// Pushes an arbitrarily-sized unsigned int literal,
    /// possibly making use of [`Intrinsic::AggregateInt`].
    /// See also [`int_literal`].
    pub fn uint_literal(&mut self, mut value: u128, size: IntegerSize) {
        let mut segments = Vec::with_capacity(4);
        loop {
            segments.push((value & 0xffffffff) as u32);
            value >>= 32;
            if value == 0 {
                break;
            }
        }

        if segments.len() != 1 {
            self.instructions
                .push(Intrinsic::AggregateLiteral(segments.len() as _).into());
        }

        for segment in segments.into_iter().rev() {
            self.instructions.push(super::Instr::UConst(segment, size));
        }
    }
}
