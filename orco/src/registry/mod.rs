use indexland::{Idx, IndexVec};

/// Holds everything that is required for module communication
#[derive(Clone, Debug, Default)]
pub struct Registry {
    /// Symbols
    symbols: std::collections::HashMap<String, Symbol>,
    /// Functions declared
    functions: IndexVec<FunctionId, Signature>,
}

impl Registry {
    /// Same as [`Default::default`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get symbol from path
    pub fn get_symbol(&self, path: &str) -> Option<Symbol> {
        self.symbols.get(path).copied()
    }

    /// Declare a function
    pub fn declare_fn(&mut self, path: String, signature: Signature) -> FunctionId {
        let fid = self.functions.push_get_id(signature);
        self.symbols.insert(path, Symbol::Function(fid));
        fid
    }

    /// Get function by id
    pub fn get_fn(&self, id: FunctionId) -> &Signature {
        &self.functions[id]
    }
}

/// Function signature
#[derive(Clone, Debug, Default)]
pub struct Signature {
    /// Function parameters
    pub params: Vec<Parameter>,
}

impl Signature {
    /// Create a new signature
    pub fn new(params: Vec<Parameter>) -> Self {
        Self { params }
    }
}

/// Function parameter, stores type and optionally name
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Parameter {
    /// Optional parameter name
    pub name: Option<String>,
    /// Parameter type
    pub ty: Type,
}

impl Parameter {
    /// Create a new parameter with a name.
    /// Also see [`Parameter::unnamed`]
    pub fn new(name: String, ty: Type) -> Self {
        Self {
            name: Some(name),
            ty,
        }
    }

    /// Create a new parameter without a name.
    /// Also see [`Parameter::new`]
    pub fn unnamed(ty: Type) -> Self {
        Self { name: None, ty }
    }
}

/// Types
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
    /// Signed integer type with set bit width
    Int(u16),
    /// Unsigned integer type with set bit width
    Unsigned(u16),
}

/// Symbol stored
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Symbol {
    /// Function
    Function(FunctionId),
}

/// Universal function ID
#[derive(Idx)]
pub struct FunctionId(pub usize);
