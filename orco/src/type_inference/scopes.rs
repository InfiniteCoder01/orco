use super::*;

/// Scope
pub type Scope = std::collections::HashMap<Span, SymbolReference>;

impl TypeInference<'_> {
    /// Start a new scope
    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    /// End the current scope
    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    /// Get the current scope
    pub fn current_scope(&self) -> &Scope {
        self.scopes.last().unwrap_or(&self.global_scope)
    }

    /// Get a mutable reference to the current scope
    pub fn current_scope_mut(&mut self) -> &mut Scope {
        self.scopes.last_mut().unwrap_or(&mut self.global_scope)
    }

    /// Try to find a symbol
    pub fn get_symbol(&self, name: &Span) -> Option<SymbolReference> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol.clone());
            }
        }
        self.global_scope.get(name).cloned()
    }

    /// Get a new variable id (just a counter)
    pub fn new_variable_id(&mut self) -> ir::expression::variable_declaration::VariableId {
        let id = self.next_variable_id;
        self.next_variable_id += 1;
        id
    }
}
