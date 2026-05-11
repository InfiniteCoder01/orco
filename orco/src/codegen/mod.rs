//! Code generation APIs, used to actually define functions and generate code.
use crate::types::IntegerSize;
use crate::{Symbol, Type};

/// Implementations of codegen features
pub mod impls;

/// Variable is a mutable storage, either in RAM or CPU registers
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Variable(pub usize);

/// Values are immutable results of operations. They can't be reused
/// unless stored in temproary variables, see [`BodyCodegen::mk_tmp`]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Value(pub usize);

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

/// Trait for generating code within a function
pub trait BodyCodegen {
    /// Leave a comment. Mainly for source2source backends
    fn comment(&mut self, comment: &str) {
        let _ = comment;
    }

    /// Get type of the value. Takes in [`Value::0`] to not consume the value
    fn type_of(&self, id: usize) -> Type;

    /// Declare a variable, see [Variable]
    fn declare_var(&mut self, ty: Type) -> Variable;
    /// Get the variable representing an argument
    fn arg_var(&self, idx: usize) -> Variable;

    /// Assign a value into a place, which makes it reusable
    fn assign(&mut self, target: Place, value: Value);
    /// Makes a temproary variable and assigns the value to it. Utility function
    fn mk_tmp(&mut self, value: Value) -> Variable {
        let tmp = self.declare_var(self.type_of(value.0));
        self.assign(tmp.into(), value);
        tmp
    }

    /// Signed integer constant
    fn iconst(&mut self, value: i128, size: IntegerSize) -> Value;
    /// Unsigned integer constant
    fn uconst(&mut self, value: u128, size: IntegerSize) -> Value;
    /// Float constant
    fn fconst(&mut self, value: f64, size: u16) -> Value;
    /// Bool constant
    fn bconst(&mut self, value: bool) -> Value;

    /// Read value from a [Place]
    fn read(&mut self, place: Place) -> Value;
    /// Get memory address of a [Place]
    fn reference(&mut self, place: Place) -> Value;

    /// Call a function (or an intrinsic)
    fn call(&mut self, func: Value, args: Vec<Value>) -> Option<Value>;

    /// Return a value from the current function.
    fn return_(&mut self, value: Option<Value>);

    /// Get intrinsic functions, see [Intrinsics]
    fn intrinsics(&mut self) -> impl Intrinsics + '_ {
        impls::Unimplemented
    }

    /// Get arbitrary control flow instructions, see [ACFCodegen]
    fn acf(&mut self) -> impl ACFCodegen + '_ {
        impls::Unimplemented
    }
}

/// Interface providing intrinsic function implementations.
pub trait Intrinsics {
    /// Integer/float addition
    #[allow(unused_variables)]
    fn add(&mut self, a: Value, b: Value) -> Value {
        unimplemented!("add operation");
    }

    /// Integer/float multiplication
    #[allow(unused_variables)]
    fn mul(&mut self, a: Value, b: Value) -> Value {
        unimplemented!("mul operation");
    }
}

/// A label ID. See [`ACFCodegen::label`]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Label(pub usize);

/// Arbitrary control flow instructions, such as jumps.
/// Warning: Not all codegens implement arbitrary control flow
pub trait ACFCodegen {
    /// Allocates a label to be placed later
    fn alloc_label(&mut self) -> Label;

    /// Places a label in the current position.
    fn label(&mut self, label: Label);

    /// Jump to a label.
    /// See [`ACFCodegen::label`]
    fn jump(&mut self, label: Label);

    /// Jumps if condition is true.
    /// See [`ACFCodegen::label`]
    fn cjump(&mut self, condition: Value, label: Label);
}

/// Interface for generating actual code.
/// All the items defined must be declared using [crate::DeclarationBackend] first.
pub trait CodegenBackend: Sync {
    /// Define a function
    fn function(&self, name: Symbol) -> impl BodyCodegen;
}
