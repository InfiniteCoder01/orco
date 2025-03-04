/// Intrinsics in OrCo are basic building blocks, functions
/// that are necessary to make programs, as every single action
/// is a function.

/// This struct holds type inference state.
/// as a mutable reference and gets filled with
/// type variables, on the second pass it's passed
/// as an immutable reference and is used to
/// finalise the types and send out diagnostics
pub struct TypeInferenceContext {
    /// See [intrinsics]
    pub intrinsics: intrinsics::Intrinsics,
    /// Scopes, see [Scope]
    pub scopes: Vec<Scope>,
}

impl TypeInferenceContext {
    /// Create a new blank type inference context
    pub fn new() -> Self {
        Self {
            intrinsics: intrinsics::Intrinsics::new(),
            scopes: Vec::new(),
        }
    }

    /// Call when starting to generate a new function

    /// Call once done with the current function
    pub fn exit_function(&mut self) {
        self.scopes.pop();
        self.intrinsics.r#return.take();
    }

    /// Resolve a variable, starting from current scope and going up
    pub fn resolve_variable(&self, name: &str) -> Option<crate::ArcLock<Variable>> {
        for scope in self.scopes.iter().rev() {
            if let Some(variable) = scope.get(name) {
                return Some(variable.clone());
            }
        }
        None
    }
}

impl Default for TypeInferenceContext {
    fn default() -> Self {
        Self::new()
    }
}

/// A scope contains variables (see [Variable]) and helps resolving them
pub type Scope = std::collections::HashMap<String, crate::ArcLock<Variable>>;

/// A regular variable
#[derive(Clone, Default)]
pub struct Variable {
    /// Optional variable name
    pub name: Option<String>,
    /// Variable type
    pub r#type: crate::Type,
}

impl Variable {
    pub fn new(name: Option<String>, r#type: crate::Type) -> Self {
        Self { name, r#type }
                .unwrap_or("unnamed variable"),
            self.r#type
        )
    }
}
