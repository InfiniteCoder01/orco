use super::Symbol;

/// Variable is a mutable storage, either in RAM or CPU registers
/// Arguments are declared as variables before codegen, and so they
/// can be accessed using `Variable(<zero-based argument index>)`
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Variable(pub usize);

impl std::fmt::Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "_{}", self.0)
    }
}

/// Values are immutable results of operations. They can't be reused
/// unless stored in temporary variables, see [`BodyCodegen::mk_tmp`]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Value(pub usize);

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{}>", self.0)
    }
}

/// A variable or symbol with projection (aka field access, dereferences, etc.)
#[derive(Debug, PartialEq, PartialOrd)]
pub enum Place {
    /// Just variable access
    Variable(Variable),
    /// Global symbol access
    Global(Symbol),
    /// Pointer dereference
    Deref(Value),
    /// Field access, using 0-based field index
    Field(Box<Place>, usize),
}

impl Place {
    /// A helper function to create [`Self::Field`]
    pub fn field(self, index: usize) -> Self {
        Self::Field(Box::new(self), index)
    }
}

impl From<Variable> for Place {
    fn from(value: Variable) -> Self {
        Self::Variable(value)
    }
}

impl Variable {
    /// Quickly convert a variable to [Place]
    pub fn place(self) -> Place {
        self.into()
    }
}

impl std::fmt::Display for Place {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Place::Variable(var) => write!(f, "{var}"),
            Place::Global(name) => write!(f, "{name}"),
            Place::Deref(value) => write!(f, "*{value}"),
            Place::Field(place, idx) => write!(f, "{place}._{idx}"),
        }
    }
}
