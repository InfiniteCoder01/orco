use crate::types::IntegerSize;

/// Intrinsics are operations built into the compier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Intrinsic {
    /// Adds two numbers. Ints and floats supported.
    Add,
    /// Subtracts two numbers. Ints and floats supported.
    Sub,
    /// Subtracts two numbers. Ints and floats supported.
    Mul,
    /// Subtracts two numbers. Ints and floats supported.
    Div,
    /// Subtracts two numbers. Ints and floats supported.
    Mod,
    /// Compares two arbitrary values. Any type supported,
    /// pointers will be compared by address.
    Eq,
    /// Constructs a bigger integer from smaller literals,
    /// useful for large constants. Type is inherited from the literals.
    /// Only argument is the number of literals to bitwise concatenate.
    AggregateInt(u8),
}

impl Intrinsic {
    /// Returns the number of arguments this intrinsic requires.
    pub fn arg_count(self) -> u32 {
        match self {
            Self::Add => 2,
            Self::Sub => 2,
            Self::Mul => 2,
            Self::Div => 2,
            Self::Mod => 2,
            Self::Eq => 2,
            Self::AggregateInt(count) => count as _,
        }
    }

    /// Weather debug display should use infix notation for this intrinsic.
    pub fn infix(self) -> bool {
        match self {
            Self::Add => true,
            Self::Sub => true,
            Self::Mul => true,
            Self::Div => true,
            Self::Mod => true,
            Self::Eq => true,
            Self::AggregateInt(..) => false,
        }
    }

    /// For some intrinsics yields their return type,
    /// for others type must be derived from the arguments.
    pub fn type_override(self) -> Option<crate::Type> {
        use crate::Type;
        Some(match self {
            Intrinsic::Eq => Type::Bool,
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
            Self::Mod => write!(f, "%"),
            Self::Eq => write!(f, "="),
            Self::AggregateInt(..) => {
                write!(f, "int")
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
                .push(Intrinsic::AggregateInt(segments.len() as _).into());
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
                .push(Intrinsic::AggregateInt(segments.len() as _).into());
        }

        for segment in segments.into_iter().rev() {
            self.instructions.push(super::Instr::UConst(segment, size));
        }
    }
}
